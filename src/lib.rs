pub mod models;
pub mod schema;

#[macro_use]
extern crate diesel;
extern crate dotenv;

use diesel::pg::PgConnection;
use diesel::prelude::*;
use schema::{sentences, todos};
mod book_todo_handler;
mod ex_nihilo_handler;
pub mod ortho;
mod ortho_todo_handler;
mod pair_todo_handler;
mod sentence_todo_handler;
mod up_handler;
mod up_helper;
mod up_on_ortho_found_handler;
pub mod web_helper;
pub mod worker_helper;

use crate::schema::pairs::table as pairs;
use crate::{models::NewTodo, schema::books::dsl::books};
use diesel::query_dsl::filter_dsl::FilterDsl;
use diesel::query_dsl::methods::SelectDsl;
use dotenv::dotenv;
use models::Book;
use std::collections::HashSet;
use std::{
    collections::hash_map::DefaultHasher,
    env,
    hash::{Hash, Hasher},
};

pub fn establish_connection() -> PgConnection {
    dotenv().ok();

    let database_url = env::var("DATABASE_URL").expect("DATABASE_URL must be set");
    PgConnection::establish(&database_url)
        .unwrap_or_else(|_| panic!("Error connecting to {}", database_url))
}

fn create_todo_entry(
    conn: &PgConnection,
    to_insert: &[NewTodo],
) -> Result<(), diesel::result::Error> {
    if !to_insert.is_empty() {
        diesel::insert_into(todos::table)
            .values(to_insert)
            .execute(conn)?;
    }
    Ok(())
}

pub fn string_to_signed_int(t: &str) -> i64 {
    let mut hasher = DefaultHasher::new();
    t.hash(&mut hasher);
    hasher.finish() as i64
}

fn project_forward(
    conn: Option<&PgConnection>,
    from: &str,
) -> Result<HashSet<String>, anyhow::Error> {
    let seconds_vec: Vec<String> = diesel::QueryDsl::select(
        diesel::QueryDsl::filter(pairs, schema::pairs::first_word.eq(from)),
        crate::schema::pairs::second_word,
    )
    .load(conn.expect("do not pass a test dummy in production"))?;

    let seconds = HashSet::from_iter(seconds_vec);
    Ok(seconds)
}

fn project_backward(
    conn: Option<&PgConnection>,
    from: &str,
) -> Result<HashSet<String>, anyhow::Error> {
    let firsts_vec: Vec<String> = SelectDsl::select(
        QueryDsl::filter(pairs, schema::pairs::second_word.eq(from)),
        crate::schema::pairs::first_word,
    )
    .load(conn.expect("do not pass a test dummy in production"))?;

    let firsts = HashSet::from_iter(firsts_vec);
    Ok(firsts)
}
