pub mod models;
pub mod schema;

#[macro_use]
extern crate diesel;
extern crate dotenv;

use diesel::pg::PgConnection;
use diesel::prelude::*;
use schema::{sentences, todos};
pub mod worker_helper;
pub mod web_helper;
mod book_todo_handler;
mod sentence_todo_handler;
mod pair_todo_handler;

use crate::{
    models::{NewTodo},
    schema::books::dsl::books,
};
use dotenv::dotenv;
use models::{Book};
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
    to_insert: &Vec<NewTodo>,
) -> Result<(), diesel::result::Error> {
    if to_insert.len() > 0 {
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
