use std::collections::HashSet;

use diesel::{QueryDsl, RunQueryDsl};

use crate::{
    create_todo_entry, establish_connection, get_hashes_of_pairs_with_words_in, insert_orthotopes,
    models::{NewOrthotope, NewTodo, Orthotope, Todo},
    ortho::Ortho,
    over_on_ortho_found_handler,
    schema::{
        self,
        orthotopes::{self, id},
    },
    up_on_ortho_found_handler,
};

pub(crate) fn handle_ortho_todo_up(todo: crate::models::Todo) -> Result<(), anyhow::Error> {
    let conn = establish_connection();
    conn.build_transaction().serializable().run(|| {
        let new_todos = vec![
            NewTodo {
                domain: "ortho_up_forward".to_string(),
                other: todo.other,
            },
            NewTodo {
                domain: "ortho_up_back".to_string(),
                other: todo.other,
            },
        ];
        create_todo_entry(&conn, new_todos)?;
        Ok(())
    })
}

pub(crate) fn handle_ortho_todo_up_forward(todo: crate::models::Todo) -> Result<(), anyhow::Error> {
    let conn = establish_connection();
    conn.build_transaction().serializable().run(|| {
        let old_orthotope = get_orthotope(&conn, todo.other)?;
        let new_orthos = new_orthotopes_up_forward(&conn, old_orthotope)?;
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

pub(crate) fn handle_ortho_todo_up_back(todo: crate::models::Todo) -> Result<(), anyhow::Error> {
    let conn = establish_connection();
    conn.build_transaction().serializable().run(|| {
        let old_orthotope = get_orthotope(&conn, todo.other)?;
        let new_orthos = new_orthotopes_up_back(&conn, old_orthotope)?;
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

pub(crate) fn handle_ortho_todo_over_forward(
    todo: crate::models::Todo,
) -> Result<(), anyhow::Error> {
    let conn = establish_connection();
    conn.build_transaction().serializable().run(|| {
        let old_orthotope = get_orthotope(&conn, todo.other)?;
        let new_orthos = new_orthotopes_over_forward(&conn, old_orthotope)?;
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

pub(crate) fn handle_ortho_todo_over_back(todo: crate::models::Todo) -> Result<(), anyhow::Error> {
    let conn = establish_connection();
    conn.build_transaction().serializable().run(|| {
        let old_orthotope = get_orthotope(&conn, todo.other)?;
        let new_orthos = new_orthotopes_over_back(&conn, old_orthotope)?;
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

pub(crate) fn handle_ortho_todo_over(todo: crate::models::Todo) -> Result<(), anyhow::Error> {
    let conn = establish_connection();
    conn.build_transaction().serializable().run(|| {
        let new_todos = vec![
            NewTodo {
                domain: "ortho_over_forward".to_string(),
                other: todo.other,
            },
            NewTodo {
                domain: "ortho_over_back".to_string(),
                other: todo.other,
            },
        ];
        create_todo_entry(&conn, new_todos)?;
        Ok(())
    })
}

pub fn handle_ortho_todo(todo: Todo) -> Result<(), anyhow::Error> {
    let conn = establish_connection();
    conn.build_transaction().serializable().run(|| {
        let new_todos = vec![
            NewTodo {
                domain: "ortho_up".to_string(),
                other: todo.other,
            },
            NewTodo {
                domain: "ortho_over".to_string(),
                other: todo.other,
            },
        ];
        create_todo_entry(&conn, new_todos)?;
        Ok(())
    })
}

fn new_orthotopes_up_forward(
    conn: &diesel::PgConnection,
    old_orthotope: Ortho,
) -> Result<Vec<NewOrthotope>, anyhow::Error> {
    let up_orthos = up_on_ortho_found_handler::up_forward(
        Some(conn),
        old_orthotope,
        crate::get_ortho_by_origin,
        crate::project_forward,
        get_hashes_of_pairs_with_words_in,
    )?;

    let orthos = up_orthos.iter();

    let res = orthos.map(crate::ortho_to_orthotope).collect();
    Ok(res)
}

fn new_orthotopes_up_back(
    conn: &diesel::PgConnection,
    old_orthotope: Ortho,
) -> Result<Vec<NewOrthotope>, anyhow::Error> {
    let up_orthos = up_on_ortho_found_handler::up_back(
        Some(conn),
        old_orthotope,
        crate::get_ortho_by_origin,
        crate::project_backward,
        get_hashes_of_pairs_with_words_in,
    )?;

    let orthos = up_orthos.iter();

    let res = orthos.map(crate::ortho_to_orthotope).collect();
    Ok(res)
}

fn new_orthotopes_over_forward(
    conn: &diesel::PgConnection,
    old_orthotope: Ortho,
) -> Result<Vec<NewOrthotope>, anyhow::Error> {
    let over_orthos: Vec<Ortho> = over_on_ortho_found_handler::over_forward(
        Some(conn),
        old_orthotope,
        crate::get_ortho_by_origin,
        crate::phrase_exists,
        crate::project_forward_batch,
        crate::get_phrases_with_matching_hashes,
    )?;

    let orthos = over_orthos.iter();

    let res = orthos.map(crate::ortho_to_orthotope).collect();
    Ok(res)
}

fn new_orthotopes_over_back(
    conn: &diesel::PgConnection,
    old_orthotope: Ortho,
) -> Result<Vec<NewOrthotope>, anyhow::Error> {
    let over_orthos: Vec<Ortho> = over_on_ortho_found_handler::over_back(
        Some(conn),
        old_orthotope,
        crate::get_ortho_by_origin,
        crate::phrase_exists,
        crate::project_backward_batch,
        crate::get_phrases_with_matching_hashes,
    )?;

    let orthos = over_orthos.iter();

    let res = orthos.map(crate::ortho_to_orthotope).collect();
    Ok(res)
}

fn get_orthotope(conn: &diesel::PgConnection, other: i32) -> Result<Ortho, anyhow::Error> {
    use crate::diesel::ExpressionMethods;
    use crate::ortho_todo_handler::orthotopes::dsl::orthotopes;

    let result: Orthotope = orthotopes
        .filter(id.eq(other))
        .select(schema::orthotopes::all_columns)
        .first(conn)?;

    let orthotope =
        bincode::deserialize(&result.information).expect("deserialization should succeed");

    Ok(orthotope)
}
