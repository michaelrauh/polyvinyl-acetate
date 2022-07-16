use std::collections::HashSet;

use diesel::PgConnection;

use crate::diesel::ExpressionMethods;
use crate::diesel::QueryDsl;
use crate::diesel::RunQueryDsl;
use crate::ortho_to_orthotope;
use crate::phrase_ortho_handler;
use crate::schema::phrases::dsl::phrases;

use crate::schema::phrases::all_columns;
use crate::{
    create_todo_entry, establish_connection, insert_orthotopes,
    models::{NewOrthotope, NewTodo, Phrase},
    schema::phrases::id,
};

pub(crate) fn handle_phrase_todo(todo: crate::models::Todo) -> Result<(), anyhow::Error> {
    let conn = establish_connection();
    conn.build_transaction().serializable().run(|| {
        let phrase = get_phrase(&conn, todo.other)?;
        let new_orthos = new_orthotopes(&conn, phrase)?;
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

fn new_orthotopes(conn: &PgConnection, phrase: Phrase) -> Result<Vec<NewOrthotope>, anyhow::Error> {
    let orthos = phrase_ortho_handler::over(
        Some(conn),
        phrase.words,
        crate::get_ortho_by_origin,
        crate::get_ortho_by_hop,
        crate::get_ortho_by_contents,
        crate::phrase_exists,
    )?;

    let res = orthos.iter().map(ortho_to_orthotope).collect();
    Ok(res)
}

fn get_phrase(conn: &PgConnection, pk: i32) -> Result<Phrase, anyhow::Error> {
    let phrase: Phrase = phrases.filter(id.eq(pk)).select(all_columns).first(conn)?;

    Ok(phrase)
}
