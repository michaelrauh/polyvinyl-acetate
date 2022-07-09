use diesel::{QueryDsl, RunQueryDsl};

use crate::{
    create_todo_entry, establish_connection, insert_orthotopes,
    models::{NewOrthotope, NewTodo, Orthotope},
    ortho::Ortho,
    over_on_ortho_found_handler,
    schema::{
        self,
        orthotopes::{self, id},
    },
    up_helper::{self},
    up_on_ortho_found_handler, get_hashes_of_pairs_with_words_in,
};

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

fn new_orthotopes(
    conn: &diesel::PgConnection,
    old_orthotope: Ortho,
) -> Result<Vec<NewOrthotope>, anyhow::Error> {
    let up_orthos = up_on_ortho_found_handler::up(
        Some(conn),
        old_orthotope.clone(),
        crate::get_ortho_by_origin,
        up_helper::pair_exists,
        crate::project_forward,
        crate::project_backward,
        get_hashes_of_pairs_with_words_in,
    )?;

    let over_orthos: Vec<Ortho> = over_on_ortho_found_handler::over(
        Some(conn),
        old_orthotope,
        crate::get_ortho_by_origin,
        crate::phrase_exists,
        crate::project_forward,
        crate::project_backward,
    )?;

    let orthos = up_orthos.iter().chain(over_orthos.iter());

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
