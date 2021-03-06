pub mod models;
pub mod schema;

#[macro_use]
extern crate diesel;
extern crate dotenv;

use diesel::dsl::any;
use diesel::pg::PgConnection;
use diesel::prelude::*;

use schema::{sentences, todos};
mod book_todo_handler;
mod ex_nihilo_handler;
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
use crate::schema::orthotopes;
use crate::schema::pairs::table as pairs;
use crate::{models::NewTodo, schema::books::dsl::books};
use diesel::query_dsl::methods::SelectDsl;
use dotenv::dotenv;
use models::Book;
use std::collections::HashSet;
use std::{
    collections::hash_map::DefaultHasher,
    env,
    hash::{Hash, Hasher},
};

type FailableStringVecToOrthoVec =
    fn(Option<&PgConnection>, Vec<String>) -> Result<Vec<Ortho>, anyhow::Error>;
type FailableStringToOrthoVec =
    fn(Option<&PgConnection>, &str) -> Result<Vec<Ortho>, anyhow::Error>;

pub fn establish_connection() -> PgConnection {
    dotenv().ok();

    let database_url = env::var("DATABASE_URL").expect("DATABASE_URL must be set");
    PgConnection::establish(&database_url)
        .unwrap_or_else(|_| panic!("Error connecting to {}", database_url))
}

pub fn get_hashes_of_pairs_with_words_in(
    conn: Option<&PgConnection>,
    first_words: HashSet<String>,
    second_words: HashSet<String>,
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

pub fn vec_of_strings_to_signed_int(v: Vec<String>) -> i64 {
    let mut hasher = DefaultHasher::new();
    v.hash(&mut hasher);
    hasher.finish() as i64
}

pub fn string_refs_to_signed_int(l: &String, r: &String) -> i64 {
    let mut hasher = DefaultHasher::new();
    l.hash(&mut hasher);
    r.hash(&mut hasher);
    hasher.finish() as i64
}

fn project_forward(
    conn: Option<&PgConnection>,
    from: &str,
) -> Result<HashSet<String>, anyhow::Error> {
    let seconds_vec: Vec<String> = diesel::QueryDsl::select(
        diesel::QueryDsl::filter(pairs, schema::pairs::first_word.eq(from)),
        crate::schema::pairs::second_word,
    )
    .load(conn.expect("do not pass a test dummy in production"))?;

    let seconds = HashSet::from_iter(seconds_vec);
    Ok(seconds)
}

fn project_backward(
    conn: Option<&PgConnection>,
    from: &str,
) -> Result<HashSet<String>, anyhow::Error> {
    let firsts_vec: Vec<String> = SelectDsl::select(
        QueryDsl::filter(pairs, schema::pairs::second_word.eq(from)),
        crate::schema::pairs::first_word,
    )
    .load(conn.expect("do not pass a test dummy in production"))?;

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
    o: &str,
) -> Result<Vec<Ortho>, anyhow::Error> {
    use crate::schema::orthotopes::{origin, table as orthotopes};
    use diesel::query_dsl::filter_dsl::FilterDsl;
    let results: Vec<Orthotope> = SelectDsl::select(
        FilterDsl::filter(orthotopes, origin.eq(o)),
        schema::orthotopes::all_columns,
    )
    .load(conn.expect("don't use test connections in production"))?;

    let res: Vec<Ortho> = results
        .iter()
        .map(|x| bincode::deserialize(&x.information).expect("deserialization should succeed"))
        .collect();

    Ok(res)
}

pub fn ortho_to_orthotope(ortho: &Ortho) -> NewOrthotope {
    let information = bincode::serialize(&ortho).expect("serialization should work");
    let origin = ortho.get_origin();
    let hop = Vec::from_iter(ortho.get_hop());
    let contents = Vec::from_iter(ortho.get_contents());
    let info_hash = pair_todo_handler::data_vec_to_signed_int(&information);
    NewOrthotope {
        information,
        origin: origin.to_owned(),
        hop,
        contents,
        info_hash,
    }
}

fn get_ortho_by_hop(
    conn: Option<&PgConnection>,
    other_hop: Vec<String>,
) -> Result<Vec<Ortho>, anyhow::Error> {
    use crate::schema::orthotopes::{hop, table as orthotopes};
    let results: Vec<Orthotope> = SelectDsl::select(
        orthotopes.filter(hop.overlaps_with(other_hop)),
        schema::orthotopes::all_columns,
    )
    .load(conn.expect("don't use test connections in production"))?;

    let res: Vec<Ortho> = results
        .iter()
        .map(|x| bincode::deserialize(&x.information).expect("deserialization should succeed"))
        .collect();

    Ok(res)
}

fn get_ortho_by_contents(
    conn: Option<&PgConnection>,
    other_contents: Vec<String>,
) -> Result<Vec<Ortho>, anyhow::Error> {
    use crate::schema::orthotopes::{contents, table as orthotopes};
    let results: Vec<Orthotope> = SelectDsl::select(
        orthotopes.filter(contents.overlaps_with(other_contents)),
        schema::orthotopes::all_columns,
    )
    .load(conn.expect("don't use test connections in production"))?;

    let res: Vec<Ortho> = results
        .iter()
        .map(|x| bincode::deserialize(&x.information).expect("deserialization should succeed"))
        .collect();

    Ok(res)
}

pub(crate) fn phrase_exists(
    conn: Option<&PgConnection>,
    phrase: Vec<String>,
) -> Result<bool, anyhow::Error> {
    use crate::schema::phrases::dsl::phrases;
    let res: bool = diesel::select(diesel::dsl::exists(
        phrases.filter(schema::phrases::words_hash.eq(vec_of_strings_to_signed_int(phrase))),
    ))
    .get_result(conn.expect("don't use the test connection"))?;

    Ok(res)
}
