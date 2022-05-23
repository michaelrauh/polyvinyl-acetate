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
mod pair_todo_handler;
mod sentence_todo_handler;
mod ortho_todo_handler;
mod up_handler;
pub mod web_helper;
pub mod worker_helper;
mod up_helper;
mod up_on_ortho_found_handler;

use crate::{models::NewTodo, schema::books::dsl::books};
use dotenv::dotenv;
use models::Book;
use std::{
    collections::hash_map::DefaultHasher,
    env,
    hash::{Hash, Hasher},
};
use std::collections::HashSet;
use crate::schema::pairs::table as pairs;

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
    let seconds_vec: Vec<String> = diesel::QueryDsl::select(diesel::QueryDsl::filter(pairs, schema::pairs::first_word.eq(from)), crate::schema::pairs::second_word)
        .load(conn.expect("do not pass a test dummy in production"))?;

    let seconds = HashSet::from_iter(seconds_vec);
    Ok(seconds)
}
