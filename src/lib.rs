pub mod models;
pub mod schema;

#[macro_use]
extern crate diesel;
extern crate dotenv;

use diesel::dsl::any;
use diesel::pg::PgConnection;
use diesel::prelude::*;

use maplit::hashset;
use schema::{phrases, sentences, todos};
mod book_todo_handler;
pub mod ortho;
mod ortho_todo_handler;
pub mod over_on_ortho_found_handler;
mod pair_todo_handler;
pub mod phrase_ortho_handler;
pub mod phrase_todo_handler;
mod sentence_todo_handler;
mod up_handler;
mod up_helper;
mod up_on_ortho_found_handler;
pub mod web_helper;
pub mod worker_helper;

use crate::models::{NewOrthotope, Orthotope};
use crate::ortho::Ortho;
use crate::schema::orthotopes::{self};
use crate::schema::pairs::table as pairs;
use crate::{models::NewTodo, schema::books::dsl::books};
use diesel::query_dsl::methods::SelectDsl;
use models::Book;
use std::collections::{HashMap, HashSet};
use std::{
    collections::hash_map::DefaultHasher,
    env,
    hash::{Hash, Hasher},
};

type FailableWordVecToOrthoVec =
    fn(Option<&PgConnection>, Vec<Word>) -> Result<Vec<Ortho>, anyhow::Error>;
type FailableWordToOrthoVec = fn(Option<&PgConnection>, Word) -> Result<Vec<Ortho>, anyhow::Error>;

type FailableHashsetWordsToHashsetNumbers = fn(
    conn: Option<&PgConnection>,
    first_words: HashSet<Word>,
    second_words: HashSet<Word>,
) -> Result<HashSet<i64>, anyhow::Error>;

type Word = i32;

pub fn establish_connection_safe() -> Result<PgConnection, ConnectionError> {
    let database_url = env::var("DATABASE_URL").expect("DATABASE_URL must be set");
    PgConnection::establish(&database_url)
}

pub fn get_hashes_of_pairs_with_words_in(
    conn: Option<&PgConnection>,
    first_words: HashSet<Word>,
    second_words: HashSet<Word>,
) -> Result<HashSet<i64>, anyhow::Error> {
    let firsts: HashSet<i64> = diesel::QueryDsl::select(
        diesel::QueryDsl::filter(
            pairs,
            schema::pairs::first_word.eq(any(Vec::from_iter(first_words))),
        ),
        crate::schema::pairs::pair_hash,
    )
    .load(conn.expect("do not pass a test dummy in production"))?
    .iter()
    .cloned()
    .collect();

    let seconds: HashSet<i64> = diesel::QueryDsl::select(
        diesel::QueryDsl::filter(
            pairs,
            schema::pairs::second_word.eq(any(Vec::from_iter(second_words))),
        ),
        crate::schema::pairs::pair_hash,
    )
    .load(conn.expect("do not pass a test dummy in production"))?
    .iter()
    .cloned()
    .collect();

    Ok(firsts.intersection(&seconds).cloned().collect())
}

pub fn get_hashes_and_words_of_pairs_with_words_in(
    conn: Option<&PgConnection>,
    first_words: HashSet<Word>,
    second_words: HashSet<Word>,
) -> Result<(HashSet<Word>, HashSet<Word>, HashSet<i64>), anyhow::Error> {
    let firsts: HashSet<(Word, Word, i64)> = diesel::QueryDsl::select(
        diesel::QueryDsl::filter(
            pairs,
            schema::pairs::first_word.eq(any(Vec::from_iter(first_words))),
        ),
        (
            crate::schema::pairs::first_word,
            crate::schema::pairs::second_word,
            crate::schema::pairs::pair_hash,
        ),
    )
    .load(conn.expect("do not pass a test dummy in production"))?
    .iter()
    .cloned()
    .collect();

    let seconds: HashSet<(Word, Word, i64)> = diesel::QueryDsl::select(
        diesel::QueryDsl::filter(
            pairs,
            schema::pairs::second_word.eq(any(Vec::from_iter(second_words))),
        ),
        (
            crate::schema::pairs::first_word,
            crate::schema::pairs::second_word,
            crate::schema::pairs::pair_hash,
        ),
    )
    .load(conn.expect("do not pass a test dummy in production"))?
    .iter()
    .cloned()
    .collect();
    let domain: HashSet<(Word, Word, i64)> = firsts.intersection(&seconds).cloned().collect();
    let mut firsts = hashset! {};
    let mut seconds = hashset! {};
    let mut hashes = hashset! {};
    domain.into_iter().for_each(|(f, s, h)| {
        firsts.insert(f);
        seconds.insert(s);
        hashes.insert(h);
    });
    Ok((firsts, seconds, hashes))
}

pub fn get_phrases_with_matching_hashes(
    conn: Option<&PgConnection>,
    all_phrases: HashSet<i64>,
) -> Result<HashSet<i64>, anyhow::Error> {
    use crate::phrases::dsl::phrases;
    let ps: HashSet<i64> = diesel::QueryDsl::select(
        diesel::QueryDsl::filter(
            phrases,
            schema::phrases::words_hash.eq(any(Vec::from_iter(all_phrases))),
        ),
        crate::schema::phrases::words_hash,
    )
    .load(conn.expect("do not pass a test dummy in production"))?
    .iter()
    .cloned()
    .collect();

    Ok(ps)
}

fn project_forward_batch(
    conn: Option<&PgConnection>,
    from: HashSet<Word>,
) -> Result<HashSet<(Word, Word)>, anyhow::Error> {
    let seconds_vec: Vec<(Word, Word)> = diesel::QueryDsl::select(
        diesel::QueryDsl::filter(
            pairs,
            schema::pairs::first_word.eq(any(Vec::from_iter(from))),
        ),
        (
            crate::schema::pairs::first_word,
            crate::schema::pairs::second_word,
        ),
    )
    .load(conn.expect("do not pass a test dummy in production"))?;

    let seconds = HashSet::from_iter(seconds_vec);
    Ok(seconds)
}

fn project_backward_batch(
    conn: Option<&PgConnection>,
    from: HashSet<Word>,
) -> Result<HashSet<(Word, Word)>, anyhow::Error> {
    let firsts_vec: Vec<(Word, Word)> = RunQueryDsl::load(
        SelectDsl::select(
            QueryDsl::filter(
                pairs,
                schema::pairs::second_word.eq(any(Vec::from_iter(from))),
            ),
            (
                crate::schema::pairs::first_word,
                crate::schema::pairs::second_word,
            ),
        ),
        conn.expect("do not pass a test dummy in production"),
    )?;

    let firsts = HashSet::from_iter(firsts_vec);
    Ok(firsts)
}

fn create_todo_entry(
    conn: &PgConnection,
    all_todos: Vec<NewTodo>,
) -> Result<(), diesel::result::Error> {
    if all_todos.is_empty() {
        return Ok(());
    }

    let to_insert: Vec<Vec<NewTodo>> = Vec::from_iter(all_todos)
        .chunks(1000)
        .map(|x| x.to_vec())
        .collect();

    for chunk in to_insert {
        diesel::insert_into(todos::table)
            .values(chunk)
            .execute(conn)?;
    }

    Ok(())
}

pub fn string_to_signed_int(t: &str) -> i64 {
    let mut hasher = DefaultHasher::new();
    t.hash(&mut hasher);
    hasher.finish() as i64
}

pub fn vec_of_words_to_big_int(v: Vec<Word>) -> i64 {
    let mut hasher = DefaultHasher::new();
    v.hash(&mut hasher);
    hasher.finish() as i64
}

pub fn string_refs_to_signed_int(l: &str, r: &str) -> i64 {
    let mut hasher = DefaultHasher::new();
    l.hash(&mut hasher);
    r.hash(&mut hasher);
    hasher.finish() as i64
}

pub fn ints_to_big_int(l: Word, r: Word) -> i64 {
    let mut hasher = DefaultHasher::new();
    l.hash(&mut hasher);
    r.hash(&mut hasher);
    hasher.finish() as i64
}

fn project_forward(
    conn: Option<&PgConnection>,
    from: Word,
) -> Result<HashSet<Word>, anyhow::Error> {
    let seconds_vec: Vec<Word> = diesel::QueryDsl::select(
        diesel::QueryDsl::filter(pairs, schema::pairs::first_word.eq(from)),
        crate::schema::pairs::second_word,
    )
    .load(conn.expect("do not pass a test dummy in production"))?;

    let seconds = HashSet::from_iter(seconds_vec);
    Ok(seconds)
}

fn project_backward(
    conn: Option<&PgConnection>,
    from: Word,
) -> Result<HashSet<Word>, anyhow::Error> {
    let firsts_vec: Vec<Word> = RunQueryDsl::load(
        SelectDsl::select(
            QueryDsl::filter(pairs, schema::pairs::second_word.eq(from)),
            crate::schema::pairs::first_word,
        ),
        conn.expect("do not pass a test dummy in production"),
    )?;

    let firsts = HashSet::from_iter(firsts_vec);
    Ok(firsts)
}

pub fn insert_orthotopes(
    conn: &PgConnection,
    new_orthos: HashSet<NewOrthotope>,
) -> Result<Vec<Orthotope>, diesel::result::Error> {
    let to_insert: Vec<Vec<NewOrthotope>> = Vec::from_iter(new_orthos)
        .chunks(1000)
        .map(|x| x.to_vec())
        .collect();

    let mut res = vec![];
    for chunk in to_insert {
        let chunk_res: Vec<Orthotope> = diesel::insert_into(orthotopes::table)
            .values(chunk)
            .on_conflict_do_nothing()
            .get_results(conn)?;
        res.push(chunk_res);
    }
    let final_res = res.into_iter().flatten().collect();

    Ok(final_res)
}

pub fn get_ortho_by_origin(
    conn: Option<&PgConnection>,
    o: Word,
) -> Result<Vec<Ortho>, anyhow::Error> {
    use crate::schema::orthotopes::{origin, table as orthotopes};
    use diesel::query_dsl::filter_dsl::FilterDsl;
    let results: Vec<Vec<u8>> = SelectDsl::select(
        FilterDsl::filter(orthotopes, origin.eq(o)),
        schema::orthotopes::information,
    )
    .load(conn.expect("don't use test connections in production"))?;

    let res: Vec<Ortho> = results
        .iter()
        .map(|x| bincode::deserialize(x).expect("deserialization should succeed"))
        .collect();

    Ok(res)
}

pub fn get_base_ortho_by_origin(
    conn: Option<&PgConnection>,
    o: Word,
) -> Result<Vec<Ortho>, anyhow::Error> {
    use crate::schema::orthotopes::{base, origin, table as orthotopes};
    use diesel::query_dsl::filter_dsl::FilterDsl;
    let results: Vec<Vec<u8>> = SelectDsl::select(
        FilterDsl::filter(orthotopes, origin.eq(o).and(base.eq(true))),
        schema::orthotopes::information,
    )
    .load(conn.expect("don't use test connections in production"))?;

    let res: Vec<Ortho> = results
        .iter()
        .map(|x| bincode::deserialize(x).expect("deserialization should succeed"))
        .collect();

    Ok(res)
}

pub fn get_ortho_by_origin_batch(
    conn: Option<&PgConnection>,
    o: HashSet<Word>,
) -> Result<Vec<Ortho>, anyhow::Error> {
    use crate::schema::orthotopes::{origin, table as orthotopes};
    use diesel::query_dsl::filter_dsl::FilterDsl;
    let results: Vec<Vec<u8>> = SelectDsl::select(
        FilterDsl::filter(orthotopes, origin.eq(any(Vec::from_iter(o)))),
        schema::orthotopes::information,
    )
    .load(conn.expect("don't use test connections in production"))?;

    let res: Vec<Ortho> = results
        .iter()
        .map(|x| bincode::deserialize(x).expect("deserialization should succeed"))
        .collect();

    Ok(res)
}

pub fn ortho_to_orthotope(ortho: &Ortho) -> NewOrthotope {
    let information = bincode::serialize(&ortho).expect("serialization should work");
    let origin = ortho.get_origin().to_owned();
    let hop = Vec::from_iter(ortho.get_hop());
    let contents = Vec::from_iter(ortho.get_contents());
    let info_hash = pair_todo_handler::data_vec_to_signed_int(&information);
    let base = ortho.is_base();
    NewOrthotope {
        information,
        origin,
        hop,
        contents,
        info_hash,
        base,
    }
}

fn get_ortho_by_hop(
    conn: Option<&PgConnection>,
    other_hop: Vec<Word>,
) -> Result<Vec<Ortho>, anyhow::Error> {
    use crate::schema::orthotopes::{hop, table as orthotopes};
    let results: Vec<Vec<u8>> = SelectDsl::select(
        orthotopes.filter(hop.overlaps_with(other_hop)),
        schema::orthotopes::information,
    )
    .load(conn.expect("don't use test connections in production"))?;

    let res: Vec<Ortho> = results
        .iter()
        .map(|x| bincode::deserialize(x).expect("deserialization should succeed"))
        .collect();

    Ok(res)
}

fn get_ortho_by_contents(
    conn: Option<&PgConnection>,
    other_contents: Vec<Word>,
) -> Result<Vec<Ortho>, anyhow::Error> {
    use crate::schema::orthotopes::{contents, table as orthotopes};
    let results: Vec<Vec<u8>> = SelectDsl::select(
        orthotopes.filter(contents.overlaps_with(other_contents)),
        schema::orthotopes::information,
    )
    .load(conn.expect("don't use test connections in production"))?;

    let res: Vec<Ortho> = results
        .iter()
        .map(|x| bincode::deserialize(x).expect("deserialization should succeed"))
        .collect();

    Ok(res)
}

pub(crate) fn phrase_exists_db_filter(
    conn: Option<&PgConnection>,
    left: HashSet<i64>,
    right: HashSet<i64>,
) -> Result<HashSet<i64>, anyhow::Error> {
    use crate::phrases::dsl::phrases;
    let firsts: HashSet<i64> = diesel::QueryDsl::select(
        diesel::QueryDsl::filter(
            phrases,
            schema::phrases::phrase_head.eq(any(Vec::from_iter(left))),
        ),
        crate::schema::phrases::words_hash,
    )
    .load(conn.expect("do not pass a test dummy in production"))?
    .iter()
    .cloned()
    .collect();

    let seconds: HashSet<i64> = diesel::QueryDsl::select(
        diesel::QueryDsl::filter(
            phrases,
            schema::phrases::phrase_tail.eq(any(Vec::from_iter(right))),
        ),
        crate::schema::phrases::words_hash,
    )
    .load(conn.expect("do not pass a test dummy in production"))?
    .iter()
    .cloned()
    .collect();

    Ok(firsts.intersection(&seconds).cloned().collect())
}

pub(crate) fn phrase_exists_db_filter_head(
    conn: Option<&PgConnection>,
    left: HashSet<i64>,
) -> Result<HashSet<i64>, anyhow::Error> {
    use crate::phrases::dsl::phrases;
    let firsts: HashSet<i64> = diesel::QueryDsl::select(
        diesel::QueryDsl::filter(
            phrases,
            schema::phrases::phrase_head.eq(any(Vec::from_iter(left))),
        ),
        crate::schema::phrases::words_hash,
    )
    .load(conn.expect("do not pass a test dummy in production"))?
    .iter()
    .cloned()
    .collect();

    Ok(firsts)
}

pub(crate) fn phrase_exists_db_filter_tail(
    conn: Option<&PgConnection>,
    right: HashSet<i64>,
) -> Result<HashSet<i64>, anyhow::Error> {
    use crate::phrases::dsl::phrases;

    let seconds: HashSet<i64> = diesel::QueryDsl::select(
        diesel::QueryDsl::filter(
            phrases,
            schema::phrases::phrase_tail.eq(any(Vec::from_iter(right))),
        ),
        crate::schema::phrases::words_hash,
    )
    .load(conn.expect("do not pass a test dummy in production"))?
    .iter()
    .cloned()
    .collect();

    Ok(seconds)
}

fn get_relevant_vocabulary(
    conn: &PgConnection,
    words: HashSet<String>,
) -> Result<HashMap<String, Word>, diesel::result::Error> {
    let res: Vec<(String, i32)> = SelectDsl::select(
        schema::words::table.filter(schema::words::word.eq(any(Vec::from_iter(words.into_iter())))),
        (schema::words::word, schema::words::id),
    )
    .load(conn)?;

    Ok(res.into_iter().collect())
}

fn get_relevant_vocabulary_reverse(
    conn: &PgConnection,
    words: HashSet<Word>,
) -> Result<HashMap<Word, String>, diesel::result::Error> {
    let res: Vec<(i32, String)> = SelectDsl::select(
        schema::words::table.filter(schema::words::id.eq(any(Vec::from_iter(words.into_iter())))),
        (schema::words::id, schema::words::word),
    )
    .load(conn)?;

    Ok(res.into_iter().collect())
}
