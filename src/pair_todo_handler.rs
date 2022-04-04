use anyhow::Error;
use std::{
    collections::{hash_map::DefaultHasher, HashSet},
    hash::{Hash, Hasher},
};

use crate::ortho::Ortho;
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
    diesel::{ExpressionMethods, query_dsl::select_dsl::SelectDsl, RunQueryDsl},
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
    let ex_nihilo_orthos: Vec<Ortho> = ex_nihilo(
        Some(conn),
        &pair.first_word,
        &pair.second_word,
        project_forward,
        project_backward,
    )?;
    let res = ex_nihilo_orthos.iter().map(ortho_to_orthotope).collect();
    Ok(res)
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

pub fn data_vec_to_signed_int(x: &[u8]) -> i64 {
    let mut hasher = DefaultHasher::new();
    x.hash(&mut hasher);
    hasher.finish() as i64
}

fn ex_nihilo(
    conn: Option<&PgConnection>,
    first: &str,
    second: &str,
    forward: fn(Option<&PgConnection>, &str) -> Result<HashSet<String>, anyhow::Error>,
    backward: fn(Option<&PgConnection>, &str) -> Result<HashSet<String>, anyhow::Error>,
) -> Result<Vec<Ortho>, anyhow::Error> {
    let mut res = vec![];
    ffbb_search(conn, first, second, forward, backward, &mut res)?;
    fbbf_search(conn, first, second, forward, backward, &mut res)?;
    Ok(res)
}

fn ffbb_search(
    conn: Option<&PgConnection>,
    a: &str,
    b: &str,
    forward: fn(Option<&PgConnection>, &str) -> Result<HashSet<String>, Error>,
    backward: fn(Option<&PgConnection>, &str) -> Result<HashSet<String>, anyhow::Error>,
    res: &mut Vec<Ortho>,
) -> Result<(), anyhow::Error> {
    for d in forward(conn, b)? {
        for c in backward(conn, &d)? {
            if b != c && backward(conn, &c)?.contains(a) {
                res.push(Ortho::new(
                    a.to_string(),
                    b.to_string(),
                    c.clone(),
                    d.clone(),
                ))
            }
        }
    }

    Ok(())
}

fn fbbf_search(
    conn: Option<&PgConnection>,
    b: &str,
    d: &str,
    forward: fn(Option<&PgConnection>, &str) -> Result<HashSet<String>, Error>,
    backward: fn(Option<&PgConnection>, &str) -> Result<HashSet<String>, anyhow::Error>,
    res: &mut Vec<Ortho>,
) -> Result<(), anyhow::Error> {
    for c in backward(conn, d)? {
        if b != c {
            for a in backward(conn, &c)? {
                if forward(conn, &a)?.contains(b) {
                    res.push(Ortho::new(
                        a.clone(),
                        b.to_string(),
                        c.clone(),
                        d.to_string(),
                    ))
                }
            }
        }
    }

    Ok(())
}

fn project_forward(conn: Option<&PgConnection>, from: &str) -> Result<HashSet<String>, anyhow::Error> {
    let seconds_vec: Vec<String> = pairs
        .filter(schema::pairs::first_word.eq(from))
        .select(crate::schema::pairs::second_word)
        .load(conn.expect("do not pass a test dummy in production"))?;

    let seconds = HashSet::from_iter(seconds_vec);
    Ok(seconds)
}

fn project_backward(conn: Option<&PgConnection>, from: &str) -> Result<HashSet<String>, anyhow::Error> {
    let firsts_vec: Vec<String> = pairs
        .filter(schema::pairs::second_word.eq(from))
        .select(crate::schema::pairs::first_word)
        .load(conn.expect("do not pass a test dummy in production"))?;

    let firsts = HashSet::from_iter(firsts_vec);
    Ok(firsts)
}

fn insert_orthotopes(
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

#[cfg(test)]
mod tests {
    use std::collections::HashSet;
    use diesel::PgConnection;

    use crate::ortho::Ortho;

    use super::ex_nihilo;

    fn fake_forward(_conn: Option<&PgConnection>, from: &str) -> Result<HashSet<String>, anyhow::Error> {
        let mut res = HashSet::default();
        if from == &"a".to_string() {
            res.insert("b".to_string());
            res.insert("c".to_string());
            Ok(res)
        } else {
            res.insert("d".to_string());
            Ok(res)
        }
    }

    fn fake_backward(_conn: Option<&PgConnection>, from: &str) -> Result<HashSet<String>, anyhow::Error> {
        let mut res = HashSet::default();
        if from == &"d".to_string() {
            res.insert("b".to_string());
            res.insert("c".to_string());
            Ok(res)
        } else {
            res.insert("a".to_string());
            Ok(res)
        }
    }

    #[test]
    fn it_creates_ex_nihilo_ffbb() {
        let actual = ex_nihilo(
            None,
            &"a".to_string(),
            &"b".to_string(),
            fake_forward,
            fake_backward,
        )
        .unwrap();
        let expected = Ortho::new(
            "a".to_string(),
            "b".to_string(),
            "c".to_string(),
            "d".to_string(),
        );

        assert_eq!(actual, vec![expected])
    }

    #[test]
    fn it_creates_ex_nihilo_fbbf() {
        let actual = ex_nihilo(
            None,
            &"b".to_string(),
            &"d".to_string(),
            fake_forward,
            fake_backward,
        )
        .unwrap();
        let expected = Ortho::new(
            "a".to_string(),
            "b".to_string(),
            "c".to_string(),
            "d".to_string(),
        );

        assert_eq!(actual, vec![expected])
    }
}