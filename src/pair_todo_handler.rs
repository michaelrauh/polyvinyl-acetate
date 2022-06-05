use std::{
    collections::hash_map::DefaultHasher,
    hash::{Hash, Hasher},
};

use crate::insert_orthotopes;
use crate::{
    create_todo_entry,
    diesel::query_dsl::filter_dsl::FilterDsl,
    ex_nihilo_handler,
    models::{NewOrthotope, NewTodo},
    schema::pairs::{dsl::pairs, id},
    up_handler, up_helper,
};
use crate::{
    diesel::{query_dsl::select_dsl::SelectDsl, ExpressionMethods, RunQueryDsl},
    establish_connection,
    models::{Pair, Todo},
    schema,
};
use diesel::PgConnection;

pub fn handle_pair_todo(todo: Todo) -> Result<(), anyhow::Error> {
    let conn = establish_connection();
    conn.build_transaction().serializable().run(|| {
        let pair = get_pair(&conn, todo.other)?;
        let new_orthos = new_orthotopes(&conn, pair)?;
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

fn new_orthotopes(conn: &PgConnection, pair: Pair) -> Result<Vec<NewOrthotope>, anyhow::Error> {
    let ex_nihilo_orthos = ex_nihilo_handler::ex_nihilo(
        Some(conn),
        &pair.first_word,
        &pair.second_word,
        crate::project_forward,
        crate::project_backward,
    )?;
    let nihilo_iter = ex_nihilo_orthos.iter();
    let up_orthos = up_handler::up(
        Some(conn),
        &pair.first_word,
        &pair.second_word,
        crate::get_ortho_by_origin,
        crate::get_ortho_by_hop,
        crate::get_ortho_by_contents,
        up_helper::pair_exists,
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
