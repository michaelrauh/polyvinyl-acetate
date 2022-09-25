use std::collections::HashSet;

use diesel::{QueryDsl, RunQueryDsl, PgConnection};

use crate::{
    create_todo_entry, get_hashes_of_pairs_with_words_in,
    insert_orthotopes,
    models::{NewOrthotope, NewTodo, Todo},
    ortho::Ortho,
    over_on_ortho_found_handler,
    schema::{
        self,
        orthotopes::{self, id},
    },
    up_on_ortho_found_handler,
};

#[tracing::instrument(level = "info", skip(pool))]
pub(crate) fn handle_ortho_todo_up(todo: crate::models::Todo, pool: diesel::r2d2::Pool<diesel::r2d2::ConnectionManager<PgConnection>>) -> Result<(), anyhow::Error> {
    let conn = pool.get()?;
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

#[tracing::instrument(level = "info", skip(pool))]
pub(crate) fn handle_ortho_todo_up_forward(todo: crate::models::Todo, pool: diesel::r2d2::Pool<diesel::r2d2::ConnectionManager<PgConnection>>) -> Result<(), anyhow::Error> {
    let conn = pool.get()?;
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

#[tracing::instrument(level = "info", skip(pool))]
pub(crate) fn handle_ortho_todo_up_back(todo: crate::models::Todo, pool: diesel::r2d2::Pool<diesel::r2d2::ConnectionManager<PgConnection>>) -> Result<(), anyhow::Error> {
    let conn = pool.get()?;
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

#[tracing::instrument(level = "info", skip(pool))]
pub(crate) fn handle_ortho_todo_over_forward(
    todo: crate::models::Todo, pool: diesel::r2d2::Pool<diesel::r2d2::ConnectionManager<PgConnection>>
) -> Result<(), anyhow::Error> {
    let conn = pool.get()?;
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

#[tracing::instrument(level = "info", skip(pool))]
pub(crate) fn handle_ortho_todo_over_back(todo: crate::models::Todo, pool: diesel::r2d2::Pool<diesel::r2d2::ConnectionManager<PgConnection>>) -> Result<(), anyhow::Error> {
    let conn = pool.get()?;
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

#[tracing::instrument(level = "info", skip(pool))]
pub(crate) fn handle_ortho_todo_over(todo: crate::models::Todo, pool: diesel::r2d2::Pool<diesel::r2d2::ConnectionManager<PgConnection>>) -> Result<(), anyhow::Error> {
    let conn = pool.get()?;
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
#[tracing::instrument(level = "info", skip(pool))]
pub fn handle_ortho_todo(todo: Todo, pool: diesel::r2d2::Pool<diesel::r2d2::ConnectionManager<PgConnection>>) -> Result<(), anyhow::Error> {
    let conn = pool.get()?;
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

#[tracing::instrument(level = "info", skip(conn))]
fn new_orthotopes_up_forward(
    conn: &diesel::PgConnection,
    old_orthotope: Ortho,
) -> Result<Vec<NewOrthotope>, anyhow::Error> {
    let up_orthos = up_on_ortho_found_handler::up_forward(
        Some(conn),
        old_orthotope,
        crate::get_ortho_by_origin_batch,
        crate::project_forward,
        get_hashes_of_pairs_with_words_in,
    )?;

    let orthos = up_orthos.iter();

    let res = orthos.map(crate::ortho_to_orthotope).collect();
    Ok(res)
}

#[tracing::instrument(level = "info", skip(conn))]
fn new_orthotopes_up_back(
    conn: &diesel::PgConnection,
    old_orthotope: Ortho,
) -> Result<Vec<NewOrthotope>, anyhow::Error> {
    let up_orthos = up_on_ortho_found_handler::up_back(
        Some(conn),
        old_orthotope,
        crate::get_ortho_by_origin_batch,
        crate::project_backward,
        get_hashes_of_pairs_with_words_in,
    )?;

    let orthos = up_orthos.iter();

    let res = orthos.map(crate::ortho_to_orthotope).collect();
    Ok(res)
}

#[tracing::instrument(level = "info", skip(conn))]
fn new_orthotopes_over_forward(
    conn: &diesel::PgConnection,
    old_orthotope: Ortho,
) -> Result<Vec<NewOrthotope>, anyhow::Error> {
    let over_orthos: Vec<Ortho> = over_on_ortho_found_handler::over_forward(
        Some(conn),
        old_orthotope,
        crate::get_ortho_by_origin_batch,
        crate::project_forward_batch,
        crate::get_phrases_with_matching_hashes,
        crate::phrase_exists_db_filter_head,
    )?;

    let orthos = over_orthos.iter();

    let res = orthos.map(crate::ortho_to_orthotope).collect();
    Ok(res)
}

#[tracing::instrument(level = "info", skip(conn))]
fn new_orthotopes_over_back(
    conn: &diesel::PgConnection,
    old_orthotope: Ortho,
) -> Result<Vec<NewOrthotope>, anyhow::Error> {
    let over_orthos: Vec<Ortho> = over_on_ortho_found_handler::over_back(
        Some(conn),
        old_orthotope,
        crate::get_ortho_by_origin_batch,
        crate::project_backward_batch,
        crate::get_phrases_with_matching_hashes,
        crate::phrase_exists_db_filter_tail,
    )?;

    let orthos = over_orthos.iter();

    let res = orthos.map(crate::ortho_to_orthotope).collect();
    Ok(res)
}

#[tracing::instrument(level = "info", skip(conn))]
fn get_orthotope(conn: &diesel::PgConnection, other: i32) -> Result<Ortho, anyhow::Error> {
    use crate::diesel::ExpressionMethods;
    use crate::ortho_todo_handler::orthotopes::dsl::orthotopes;

    let result: Vec<u8> = orthotopes
        .filter(id.eq(other))
        .select(schema::orthotopes::information)
        .first(conn)?;

    let orthotope = bincode::deserialize(&result).expect("deserialization should succeed");

    Ok(orthotope)
}
