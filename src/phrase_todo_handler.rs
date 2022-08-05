use std::collections::HashSet;

use diesel::PgConnection;

use crate::diesel::ExpressionMethods;
use crate::diesel::QueryDsl;
use crate::diesel::RunQueryDsl;
use crate::models::Todo;
use crate::ortho_to_orthotope;
use crate::phrase_ortho_handler;
use crate::schema::phrases::dsl::phrases;
use crate::Word;

use crate::{
    create_todo_entry, establish_connection_safe, insert_orthotopes,
    models::{NewOrthotope, NewTodo},
    schema::phrases::id,
};

pub(crate) fn handle_phrase_todo_origin(todo: crate::models::Todo) -> Result<(), anyhow::Error> {
    let conn = establish_connection_safe()?;
    conn.build_transaction().serializable().run(|| {
        let phrase = get_phrase(&conn, todo.other)?;
        let new_orthos = new_orthotopes_by_origin(&conn, phrase)?;
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

pub(crate) fn handle_phrase_todo_hop(todo: crate::models::Todo) -> Result<(), anyhow::Error> {
    let conn = establish_connection_safe()?;
    conn.build_transaction().serializable().run(|| {
        let phrase = get_phrase(&conn, todo.other)?;
        let new_orthos = new_orthotopes_by_hop(&conn, phrase)?;
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

pub(crate) fn handle_phrase_todo_contents(todo: crate::models::Todo) -> Result<(), anyhow::Error> {
    let conn = establish_connection_safe()?;
    conn.build_transaction().serializable().run(|| {
        let phrase = get_phrase(&conn, todo.other)?;
        let new_orthos = new_orthotopes_by_contents(&conn, phrase)?;
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

pub fn handle_phrase_todo(todo: Todo) -> Result<(), anyhow::Error> {
    let conn = establish_connection_safe()?;
    conn.build_transaction().serializable().run(|| {
        let new_todos = vec![
            NewTodo {
                domain: "phrase_by_origin".to_string(),
                other: todo.other,
            },
            NewTodo {
                domain: "phrase_by_hop".to_string(),
                other: todo.other,
            },
            NewTodo {
                domain: "phrase_by_contents".to_string(),
                other: todo.other,
            },
        ];
        create_todo_entry(&conn, new_todos)?;
        Ok(())
    })
}

fn new_orthotopes_by_origin(
    conn: &PgConnection,
    phrase: Vec<Word>,
) -> Result<Vec<NewOrthotope>, anyhow::Error> {
    let orthos = phrase_ortho_handler::over_by_origin(
        Some(conn),
        phrase,
        crate::get_ortho_by_origin,
        crate::phrase_exists_db_filter,
    )?;

    let res = orthos.iter().map(ortho_to_orthotope).collect();
    Ok(res)
}
fn new_orthotopes_by_hop(
    conn: &PgConnection,
    phrase: Vec<Word>,
) -> Result<Vec<NewOrthotope>, anyhow::Error> {
    let orthos = phrase_ortho_handler::over_by_hop(
        Some(conn),
        phrase,
        crate::get_ortho_by_hop,
        crate::phrase_exists_db_filter,
    )?;

    let res = orthos.iter().map(ortho_to_orthotope).collect();
    Ok(res)
}
fn new_orthotopes_by_contents(
    conn: &PgConnection,
    phrase: Vec<Word>,
) -> Result<Vec<NewOrthotope>, anyhow::Error> {
    let orthos = phrase_ortho_handler::over_by_contents(
        Some(conn),
        phrase,
        crate::get_ortho_by_contents,
        crate::phrase_exists,
    )?;

    let res = orthos.iter().map(ortho_to_orthotope).collect();
    Ok(res)
}

fn get_phrase(conn: &PgConnection, pk: i32) -> Result<Vec<Word>, anyhow::Error> {
    let phrase: Vec<Word> = phrases
        .filter(id.eq(pk))
        .select(crate::schema::phrases::words)
        .first(conn)?;

    Ok(phrase)
}
