pub mod models;
pub mod schema;

#[macro_use]
extern crate diesel;
extern crate dotenv;

use diesel::pg::PgConnection;
use diesel::prelude::*;
use schema::todos;

use crate::{
    models::{NewBook, NewTodo},
    schema::books::dsl::books,
};
use dotenv::dotenv;
use models::{Book, Todo};
use std::env;

pub fn get_todos() -> Result<Vec<Todo>, diesel::result::Error> {
    use crate::schema::todos::dsl::todos;

    let results = todos.load(&establish_connection())?;

    Ok(results)
}

pub fn establish_connection() -> PgConnection {
    dotenv().ok();

    let database_url = env::var("DATABASE_URL").expect("DATABASE_URL must be set");
    PgConnection::establish(&database_url)
        .unwrap_or_else(|_| panic!("Error connecting to {}", database_url))
}

pub fn create_book(
    conn: &PgConnection,
    title: String,
    body: String,
) -> Result<Book, diesel::result::Error> {
    conn.build_transaction().serializable().run(|| {
        let book = create_book_entry(conn, title, body)?;
        create_todo_entry(conn, book.id, "books".to_owned())?;

        Ok(book)
    })
}

fn create_todo_entry(
    conn: &PgConnection,
    fk: i32,
    domain: String,
) -> Result<Todo, diesel::result::Error> {
    diesel::insert_into(todos::table)
        .values(&NewTodo { domain, other: fk })
        .get_result(conn)
}

fn create_book_entry(
    conn: &PgConnection,
    title: String,
    body: String,
) -> Result<Book, diesel::result::Error> {
    use schema::books;

    diesel::insert_into(books::table)
        .values(&NewBook { title, body })
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
    let results: i64 = todos.count().get_result(&establish_connection())?;

    Ok(results.to_string())
}

pub fn show_depth() -> Result<String, amiquip::Error> {
    use amiquip::Connection;

    let rabbit_url = env::var("RABBIT_URL").expect("RABBIT_URL must be set");

    let mut connection = Connection::insecure_open(&rabbit_url)?;

    let channel = connection.open_channel(None)?;
    let queue = channel.queue_declare(
        "work",
        amiquip::QueueDeclareOptions {
            durable: true,
            ..amiquip::QueueDeclareOptions::default()
        },
    )?;

    let depth = queue
        .declared_message_count()
        .expect("queue must be declared non-immediate");
    Ok(depth.to_string())
}

pub fn delete_todos(todos_to_delete: Vec<Todo>) -> Result<usize, diesel::result::Error> {
    use crate::todos::dsl::todos;
    let ids = todos_to_delete.iter().map(|t| t.id);
    let f = todos.filter(schema::todos::id.eq_any(ids));
    diesel::delete(f).execute(&establish_connection())
}

pub fn handle_todo(todo: Todo) -> Result<(), anyhow::Error> {
    Ok(())
}
