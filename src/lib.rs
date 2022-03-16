pub mod models;
pub mod schema;

#[macro_use]
extern crate diesel;
extern crate dotenv;

use diesel::pg::PgConnection;
use diesel::prelude::*;

use crate::{models::{NewBook, NewTodo}, schema::books::dsl::books};
use dotenv::dotenv;
use models::{Book, Todo};
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
        .run(|| { 
            let book = create_book_entry(conn, title, body)?;
            create_todo_entry(conn, book.id, "books".to_owned())?;

            Ok(book)
        })
}

fn create_todo_entry(conn: &PgConnection, fk: i32, domain: String) -> Result<Todo, diesel::result::Error> {
    use schema::todos;

    diesel::insert_into(todos::table)
    .values(&NewTodo {
        domain: domain,
        other: fk,
    })
    .get_result(conn)
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
    let results: Vec<String> = books
        .select(schema::books::title)
        .load(&establish_connection())?;

    Ok(results.join("\n"))
}

pub fn show_todos() -> Result<String, diesel::result::Error> {
    use crate::schema::todos::dsl::todos;
    let results: i64 = todos.count()
    .get_result(&establish_connection())?;

    Ok(results.to_string())
}
