use std::{
    collections::{hash_map::DefaultHasher, BTreeMap},
    hash::{Hash, Hasher},
};

use crate::ortho::Location;
use crate::{
    create_todo_entry,
    diesel::query_dsl::filter_dsl::FilterDsl,
    models::{NewOrthotope, NewTodo, Orthotope},
    schema::{
        orthotopes,
        pairs::{dsl::pairs, id},
    },
};
use crate::{
    diesel::{query_dsl::select_dsl::SelectDsl, ExpressionMethods, RunQueryDsl},
    establish_connection,
    models::{Pair, Todo},
    schema,
};
use diesel::PgConnection;
use serde::{Deserialize, Serialize};

pub fn handle_pair_todo(todo: Todo) -> Result<(), anyhow::Error> {
    let conn = establish_connection();
    conn.build_transaction().serializable().run(|| {
        let pair = get_pair(&conn, todo.other)?;
        let new_orthos = new_orthos(pair);
        let inserted_orthos = insert_orthos(&conn, &new_orthos)?;
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

#[derive(Serialize, Deserialize, Debug, PartialEq)]
pub struct Ortho {
    pub(crate) info: BTreeMap<Location, String>,
}

fn new_orthos(pair: Pair) -> Vec<NewOrthotope> {
    let ex_nihilo_orthos: Vec<Ortho> = ex_nihilo(pair);
    ex_nihilo_orthos
        .iter()
        .map(|x| ortho_to_orthotope(x))
        .collect()
}

fn ortho_to_orthotope(ortho: &Ortho) -> NewOrthotope {
    let information = bincode::serialize(&ortho).expect("serialization should work");
    let origin = ortho.get_origin();
    let hop = Vec::from_iter(ortho.get_hop());
    let contents = Vec::from_iter(ortho.get_contents());
    let info_hash = data_vec_to_signed_int(&information);
    NewOrthotope {
        information,
        origin,
        hop,
        contents,
        info_hash,
    }
}

pub fn data_vec_to_signed_int(x: &Vec<u8>) -> i64 {
    let mut hasher = DefaultHasher::new();
    x.hash(&mut hasher);
    hasher.finish() as i64
}

fn ex_nihilo(pair: Pair) -> Vec<Ortho> {
    vec![
        Ortho::new(
            "a".to_string(),
            "b".to_string(),
            "c".to_string(),
            "d".to_string(),
        ),
        Ortho::new(
            "a".to_string(),
            "c".to_string(),
            "b".to_string(),
            "d".to_string(),
        ),
    ]
}

fn insert_orthos(
    conn: &PgConnection,
    new_orthos: &[NewOrthotope],
) -> Result<Vec<Orthotope>, diesel::result::Error> {
    diesel::insert_into(orthotopes::table)
        .values(new_orthos)
        .on_conflict_do_nothing()
        .get_results(conn)
}

fn get_pair(conn: &PgConnection, pk: i32) -> Result<Pair, anyhow::Error> {
    let pair: Pair = pairs
        .filter(id.eq(pk))
        .select(schema::pairs::all_columns)
        .first(conn)?;

    Ok(pair)
}
