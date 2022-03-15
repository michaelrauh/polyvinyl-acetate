pub mod models;
pub mod schema;

#[macro_use]
extern crate diesel;
extern crate dotenv;

use diesel::pg::PgConnection;
use diesel::prelude::*;

use crate::{models::NewBook, schema::books::dsl::books};
use dotenv::dotenv;
use models::Book;
use std::env;

pub fn establish_connection() -> PgConnection {
    dotenv().ok();

    let database_url = env::var("DATABASE_URL").expect("DATABASE_URL must be set");
    PgConnection::establish(&database_url).expect(&format!("Error connecting to {}", database_url))
}

pub fn create_book(
    conn: &PgConnection,
    title: String,
    body: String,
) -> Result<Book, diesel::result::Error> {
    conn.build_transaction()
        .serializable()
        .run(|| create_book_entry(conn, title, body))
}

fn create_book_entry(
    conn: &PgConnection,
    title: String,
    body: String,
) -> Result<Book, diesel::result::Error> {
    use schema::books;

    diesel::insert_into(books::table)
        .values(&NewBook {
            title: title,
            body: body,
        })
        .get_result(conn)
}

pub fn show_books() -> Result<String, diesel::result::Error> {
    let results: Result<Vec<String>, diesel::result::Error> = books
        .select(schema::books::title)
        .load(&establish_connection());

    results.map(|s| s.join("\n"))
}
