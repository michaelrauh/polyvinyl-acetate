use std::env;

use diesel::{PgConnection, QueryDsl, RunQueryDsl};
use crate::{Book, NewTodo, create_todo_entry, schema::{books, self}, models::NewBook, establish_connection};

pub fn create_book(
    conn: &PgConnection,
    title: String,
    body: String,
) -> Result<Book, diesel::result::Error> {
    conn.build_transaction().serializable().run(|| {
        let book = create_book_entry(conn, title, body)?;
        let to_insert = vec![NewTodo {
            domain: "books".to_owned(),
            other: book.id,
        }];
        create_todo_entry(conn, &to_insert)?;
        Ok(book)
    })
}

fn create_book_entry(
    conn: &PgConnection,
    title: String,
    body: String,
) -> Result<Book, diesel::result::Error> {
    diesel::insert_into(books::table)
        .values(&NewBook { title, body })
        .get_result(conn)
}

pub fn show_books() -> Result<String, diesel::result::Error> {
    use crate::diesel::query_dsl::select_dsl::SelectDsl;
    use crate::books;
    let results: Vec<String> = SelectDsl::select(books, schema::books::title)
        .load(&establish_connection())?;

    Ok(results.join("\n"))
}

pub fn show_todos() -> Result<String, diesel::result::Error> {
    use crate::schema::todos::dsl::todos;
    let results: i64 = todos.count().get_result(&establish_connection())?;

    Ok(results.to_string())
}

pub fn count_sentences() -> Result<String, diesel::result::Error> {
    use crate::schema::sentences::dsl::sentences;
    let results: i64 = sentences.count().get_result(&establish_connection())?;

    Ok(results.to_string())
}

pub fn count_pairs() -> Result<String, diesel::result::Error> {
    use crate::schema::pairs::dsl::pairs;
    let results: i64 = pairs.count().get_result(&establish_connection())?;

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