use std::{
    collections::{hash_map::DefaultHasher, HashSet},
    hash::{Hash, Hasher},
};

use crate::{
    create_todo_entry,
    diesel::query_dsl::filter_dsl::FilterDsl,
    ex_nihilo_handler, get_hashes_of_pairs_with_words_in,
    models::{NewOrthotope, NewTodo},
    schema::pairs::{dsl::pairs, id},
    up_handler, Word,
};
use crate::{
    diesel::{query_dsl::select_dsl::SelectDsl, ExpressionMethods, RunQueryDsl},
    establish_connection,
    models::{Pair, Todo},
    schema,
};
use crate::{insert_orthotopes, models::ExNihilo, ortho::Ortho};
use diesel::{sql_query, PgConnection};

pub fn handle_pair_todo(todo: Todo) -> Result<(), anyhow::Error> {
    let conn = establish_connection();
    conn.build_transaction().serializable().run(|| {
        let pair = get_pair(&conn, todo.other)?;
        let new_orthos = new_orthotopes(&conn, pair)?;
        let inserted_orthos = insert_orthotopes(&conn, HashSet::from_iter(new_orthos))?;
        let todos: Vec<NewTodo> = inserted_orthos
            .iter()
            .map(|s| NewTodo {
                domain: "orthotopes".to_owned(),
                other: s.id,
            })
            .collect();
        create_todo_entry(&conn, todos)?;
        Ok(())
    })
}

fn single_ffbb(
    conn: Option<&PgConnection>,
    first: Word,
    second: Word,
) -> Result<Vec<Ortho>, anyhow::Error> {
    let query = format!(
        "SELECT CD.first_word, CD.second_word
        FROM pairs CD
        INNER JOIN pairs AC ON AC.second_word=CD.first_word
        INNER JOIN pairs BD ON BD.second_word=CD.second_word AND BD.first_word<>AC.second_word
        WHERE BD.first_word='{}'
        AND AC.first_word='{}';",
        second, first
    );
    let ffbbs: Vec<ExNihilo> =
        sql_query(query).load(conn.expect("do not pass a test dummy in production"))?;

    let res = ffbbs
        .iter()
        .map(|r| {
            Ortho::new(
                first.to_owned(),
                second.to_owned(),
                r.first_word.to_owned(),
                r.second_word.to_owned(),
            )
        })
        .collect();

    Ok(res)
}

fn single_fbbf(
    conn: Option<&PgConnection>,
    first: Word,
    second: Word,
) -> Result<Vec<Ortho>, anyhow::Error> {
    let query = format!(
        "SELECT AC.first_word, AC.second_word
        FROM pairs AC
        INNER JOIN pairs AB ON AC.first_word=AB.first_word AND AB.second_word<>AC.second_word
        INNER JOIN pairs CD ON AC.second_word=CD.first_word 
        WHERE AB.second_word='{}'
        AND CD.second_word='{}';",
        first, second
    );
    let ffbbs: Vec<ExNihilo> =
        sql_query(query).load(conn.expect("do not pass a test dummy in production"))?;

    // finding ac given bd
    let res = ffbbs
        .iter()
        .map(|r| {
            Ortho::new(
                r.first_word.to_owned(),
                first.to_owned(),
                r.second_word.to_owned(),
                second.to_owned(),
            )
        })
        .collect();

    Ok(res)
}

fn new_orthotopes(conn: &PgConnection, pair: Pair) -> Result<Vec<NewOrthotope>, anyhow::Error> {
    let ex_nihilo_orthos = ex_nihilo_handler::ex_nihilo(
        Some(conn),
        pair.first_word,
        pair.second_word,
        single_ffbb,
        single_fbbf,
    )?;
    let nihilo_iter = ex_nihilo_orthos.iter();
    let up_orthos = up_handler::up(
        Some(conn),
        pair.first_word,
        pair.second_word,
        crate::get_ortho_by_origin,
        crate::get_ortho_by_hop,
        crate::get_ortho_by_contents,
        get_hashes_of_pairs_with_words_in,
    )?;
    let up_iter = up_orthos.iter();
    let both = nihilo_iter.chain(up_iter);

    let res = both.map(crate::ortho_to_orthotope).collect();
    Ok(res)
}

pub fn data_vec_to_signed_int(x: &[u8]) -> i64 {
    let mut hasher = DefaultHasher::new();
    x.hash(&mut hasher);
    hasher.finish() as i64
}

fn get_pair(conn: &PgConnection, pk: i32) -> Result<Pair, anyhow::Error> {
    let pair: Pair = pairs
        .filter(id.eq(pk))
        .select(schema::pairs::all_columns)
        .first(conn)?;

    Ok(pair)
}
