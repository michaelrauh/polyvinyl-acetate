use diesel::{QueryDsl, RunQueryDsl};

use crate::{establish_connection, ortho::Ortho, models::{NewOrthotope, NewTodo, Orthotope}, up_helper::{insert_orthotopes, self}, create_todo_entry, schema::{orthotopes::{self, id}, self}, up_on_ortho_found_handler};

pub(crate) fn handle_ortho_todo(todo: crate::models::Todo) -> Result<(), anyhow::Error> {
    let conn = establish_connection();
    conn.build_transaction().serializable().run(|| {
        let old_orthotope = get_orthotope(&conn, todo.other)?;
        let new_orthos = new_orthotopes(&conn, old_orthotope)?;
        let inserted_orthos = insert_orthotopes(&conn, &new_orthos)?;
        let todos: Vec<NewTodo> = inserted_orthos
            .iter()
            .map(|s| NewTodo {
                domain: "orthotopes".to_owned(),
                other: s.id,
            })
            .collect();
        create_todo_entry(&conn, &todos)?;
        Ok(())
    })
}

fn new_orthotopes(conn: &diesel::PgConnection, old_orthotope: Ortho) -> Result<Vec<NewOrthotope>, anyhow::Error> {
    let up_orthos = up_on_ortho_found_handler::up(
        Some(conn),
        old_orthotope,
        up_helper::get_ortho_by_origin,
        up_helper::pair_exists,
        crate::project_forward,
    )?;

    let res = up_orthos.iter().map(up_helper::ortho_to_orthotope).collect();
    Ok(res)
}

fn get_orthotope(conn: &diesel::PgConnection, other: i32) -> Result<Ortho, anyhow::Error> {
    use crate::ortho_todo_handler::orthotopes::dsl::orthotopes;
    use crate::diesel::ExpressionMethods;

    let result: Orthotope = orthotopes
        .filter(id.eq(other))
        .select(schema::orthotopes::all_columns)
        .first(conn)?;

    let orthotope = bincode::deserialize(&result.information).expect("deserialization should succeed");

    Ok(orthotope)
}