use std::{
    collections::{hash_map::DefaultHasher, HashSet},
    hash::{Hash, Hasher},
};

use crate::{
    create_todo_entry,
    diesel::query_dsl::filter_dsl::FilterDsl,
    get_hashes_and_words_of_pairs_with_words_in,
    models::{NewOrthotope, NewTodo},
    schema::pairs::{dsl::pairs, id},
    up_handler, Word,
};
use crate::{
    diesel::{query_dsl::select_dsl::SelectDsl, ExpressionMethods, RunQueryDsl},
    establish_connection_safe,
    models::Todo,
    schema,
};
use crate::{insert_orthotopes, models::ExNihilo, ortho::Ortho};
use diesel::{sql_query, PgConnection};

#[tracing::instrument(level = "info")]
pub fn handle_pair_todo_up_by_origin(todo: Todo) -> Result<(), anyhow::Error> {
    let conn = establish_connection_safe()?;
    conn.build_transaction().serializable().run(|| {
        let pair = get_pair(&conn, todo.other)?;
        let new_orthos = new_orthotopes_up_by_origin(&conn, pair)?;
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

#[tracing::instrument(level = "info")]
pub fn handle_pair_todo_up_by_contents(todo: Todo) -> Result<(), anyhow::Error> {
    let conn = establish_connection_safe()?;
    conn.build_transaction().serializable().run(|| {
        let pair = get_pair(&conn, todo.other)?;
        let new_orthos = new_orthotopes_up_by_contents(&conn, pair)?;
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

#[tracing::instrument(level = "info")]
pub fn handle_pair_todo_up_by_hop(todo: Todo) -> Result<(), anyhow::Error> {
    let conn = establish_connection_safe()?;
    conn.build_transaction().serializable().run(|| {
        let pair = get_pair(&conn, todo.other)?;
        let new_orthos = new_orthotopes_up_by_hop(&conn, pair)?;
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

#[tracing::instrument(level = "info")]
pub fn handle_pair_todo_ffbb(todo: Todo) -> Result<(), anyhow::Error> {
    let conn = establish_connection_safe()?;
    conn.build_transaction().serializable().run(|| {
        let pair = get_pair(&conn, todo.other)?;
        let new_orthos = new_orthotopes_ffbb(&conn, pair)?;
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

#[tracing::instrument(level = "info")]
pub fn handle_pair_todo_fbbf(todo: Todo) -> Result<(), anyhow::Error> {
    let conn = establish_connection_safe()?;
    conn.build_transaction().serializable().run(|| {
        let pair = get_pair(&conn, todo.other)?;
        let new_orthos = new_orthotopes_fbbf(&conn, pair)?;
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

pub fn handle_pair_todo_up(todo: Todo) -> Result<(), anyhow::Error> {
    let conn = establish_connection_safe()?;
    conn.build_transaction().serializable().run(|| {
        let new_todos = vec![
            NewTodo {
                domain: "up_by_origin".to_string(),
                other: todo.other,
            },
            NewTodo {
                domain: "up_by_hop".to_string(),
                other: todo.other,
            },
            NewTodo {
                domain: "up_by_contents".to_string(),
                other: todo.other,
            },
        ];
        create_todo_entry(&conn, new_todos)?;
        Ok(())
    })
}

#[tracing::instrument(level = "info")]
pub fn handle_pair_todo(todo: Todo) -> Result<(), anyhow::Error> {
    let conn = establish_connection_safe()?;
    conn.build_transaction().serializable().run(|| {
        let new_todos = vec![
            NewTodo {
                domain: "ex_nihilo_ffbb".to_string(),
                other: todo.other,
            },
            NewTodo {
                domain: "ex_nihilo_fbbf".to_string(),
                other: todo.other,
            },
            NewTodo {
                domain: "pair_up".to_string(),
                other: todo.other,
            },
        ];
        create_todo_entry(&conn, new_todos)?;
        Ok(())
    })
}

#[tracing::instrument(level = "info", skip(conn))]
fn single_ffbb(
    conn: &PgConnection,
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
    let ffbbs: Vec<ExNihilo> = sql_query(query).load(conn)?;

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

#[tracing::instrument(level = "info", skip(conn))]
fn single_fbbf(
    conn: &PgConnection,
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
    let ffbbs: Vec<ExNihilo> = sql_query(query).load(conn)?;

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

#[tracing::instrument(level = "info", skip(conn))]
fn new_orthotopes_up_by_origin(
    conn: &PgConnection,
    pair: (Word, Word),
) -> Result<Vec<NewOrthotope>, anyhow::Error> {
    let up_orthos = up_handler::up_by_origin(
        Some(conn),
        pair.0,
        pair.1,
        crate::get_base_ortho_by_origin,
        get_hashes_and_words_of_pairs_with_words_in,
    )?;
    let up_iter = up_orthos.iter();

    let res = up_iter.map(crate::ortho_to_orthotope).collect();
    Ok(res)
}

#[tracing::instrument(level = "info", skip(conn))]
fn new_orthotopes_up_by_hop(
    conn: &PgConnection,
    pair: (Word, Word),
) -> Result<Vec<NewOrthotope>, anyhow::Error> {
    let up_orthos = up_handler::up_by_hop(
        Some(conn),
        pair.0,
        pair.1,
        crate::get_base_ortho_by_hop,
        get_hashes_and_words_of_pairs_with_words_in,
    )?;
    let up_iter = up_orthos.iter();

    let res = up_iter.map(crate::ortho_to_orthotope).collect();
    Ok(res)
}

#[tracing::instrument(level = "info", skip(conn))]
fn new_orthotopes_up_by_contents(
    conn: &PgConnection,
    pair: (Word, Word),
) -> Result<Vec<NewOrthotope>, anyhow::Error> {
    let up_orthos = up_handler::up_by_contents(
        Some(conn),
        pair.0,
        pair.1,
        crate::get_base_ortho_by_contents,
        get_hashes_and_words_of_pairs_with_words_in,
    )?;
    let up_iter = up_orthos.iter();

    let res = up_iter.map(crate::ortho_to_orthotope).collect();
    Ok(res)
}

#[tracing::instrument(level = "info", skip(conn))]
fn new_orthotopes_ffbb(
    conn: &PgConnection,
    pair: (Word, Word),
) -> Result<Vec<NewOrthotope>, anyhow::Error> {
    let ex_nihilo_orthos = single_ffbb(conn, pair.0, pair.1)?;

    let nihilo_iter = ex_nihilo_orthos.iter();

    let res = nihilo_iter.map(crate::ortho_to_orthotope).collect();
    Ok(res)
}

#[tracing::instrument(level = "info", skip(conn))]
fn new_orthotopes_fbbf(
    conn: &PgConnection,
    pair: (Word, Word),
) -> Result<Vec<NewOrthotope>, anyhow::Error> {
    let ex_nihilo_orthos = single_fbbf(conn, pair.0, pair.1)?;
    let nihilo_iter = ex_nihilo_orthos.iter();

    let res = nihilo_iter.map(crate::ortho_to_orthotope).collect();
    Ok(res)
}

pub fn data_vec_to_signed_int(x: &[u8]) -> i64 {
    let mut hasher = DefaultHasher::new();
    x.hash(&mut hasher);
    hasher.finish() as i64
}

#[tracing::instrument(level = "info", skip(conn))]
fn get_pair(conn: &PgConnection, pk: i32) -> Result<(Word, Word), anyhow::Error> {
    let pair: (Word, Word) = pairs
        .filter(id.eq(pk))
        .select((schema::pairs::first_word, schema::pairs::second_word))
        .first(conn)?;

    Ok(pair)
}
