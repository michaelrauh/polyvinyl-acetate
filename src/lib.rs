pub mod models;
pub mod schema;

#[macro_use]
extern crate diesel;
extern crate dotenv;

use diesel::pg::PgConnection;
use diesel::prelude::*;
use schema::{sentences, todos};

use crate::{
    models::{NewBook, NewTodo},
    schema::books::dsl::books,
};
use dotenv::dotenv;
use models::{Book, NewSentence, Todo, Sentence};
use std::{
    collections::hash_map::DefaultHasher,
    env,
    hash::{Hash, Hasher},
};

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
        let to_insert = vec![NewTodo { domain: "books".to_owned(), other: book.id }];
        create_todo_entry(conn, &to_insert)?;

        Ok(book)
    })
}

fn create_todo_entry(
    conn: &PgConnection,
    to_insert: &Vec<NewTodo>,
) -> Result<Todo, diesel::result::Error> {
    diesel::insert_into(todos::table)
        .values(to_insert)
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

pub fn count_sentences() -> Result<String, diesel::result::Error> {
    use crate::schema::sentences::dsl::sentences;
    let results: i64 = sentences.count().get_result(&establish_connection())?;

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
    match todo.domain.as_str() {
        "books" => handle_book_todo(todo),
        "sentences" => { println!("dropping sentence todo!"); Ok(()) } ,
        other => { panic!("getting unexpected todo with domain: {other}") }
    }
    
}

fn handle_book_todo(todo: Todo) -> Result<(), anyhow::Error> {
    let conn = establish_connection();
    conn.build_transaction().serializable().run(|| {
        let book = get_book(&conn, todo.other)?;
        let new_sentences = split_book_to_sentences(book);
        let sentences = insert_sentences(&conn, &new_sentences)?;
        let todos = sentences.iter().map(|s| NewTodo { domain: "sentences".to_owned(), other: s.id  }).collect();
        create_todo_entry(&conn, &todos)?;
        Ok(())
    })
}

fn insert_sentences(
    conn: &PgConnection,
    sentences: &[NewSentence],
) -> Result<Vec<Sentence>, diesel::result::Error> {
    diesel::insert_into(sentences::table)
        .values(sentences)
        .on_conflict_do_nothing()
        .get_results(conn)
}

fn get_book(conn: &PgConnection, pk: i32) -> Result<Book, anyhow::Error> {
    use crate::schema::books::id;
    let book: Book = books
        .filter(id.eq(pk))
        .select(schema::books::all_columns)
        .first(conn)?;

    Ok(book)
}

fn split_book_to_sentences(book: Book) -> Vec<NewSentence> {
    book.body
        .split(|x| x == '.' || x == '!' || x == '?' || x == ';')
        .filter(|x| !x.is_empty())
        .map(|x| x.trim())
        .map(|x| x.to_string())
        .map(|sentence| {
            sentence
                .replace("-", "")
                .replace(":", "")
                .replace(",", "")
                .to_lowercase()})
        .map(|t| NewSentence {
            sentence: t.clone(),
            sentence_hash: string_to_signed_int(&t),
        })
        .collect()
}

fn string_to_signed_int(t: &str) -> i64 {
    let mut hasher = DefaultHasher::new();
    t.hash(&mut hasher);
    hasher.finish() as i64
}

#[cfg(test)]
mod tests {

    use crate::{
        models::{Book},
        split_book_to_sentences, string_to_signed_int,
    };

    #[test]
    fn it_splits_books_to_sentences() {
        let book = Book {
            title: "title".to_owned(),
            body: "Multiple words.. \n\tTwo sentences! Now,:- three; Four.".to_owned(),
            id: 5,
        };
        let actual = split_book_to_sentences(book);
        let actual_sentences: Vec<String> = actual.iter().map(|s| s.sentence.clone()).collect();
        let actual_hashes: Vec<i64> = actual.iter().map(|s| s.sentence_hash).collect();
        assert_eq!(
            actual_sentences,
            vec!["multiple words", "two sentences", "now three", "four"]
        );

        assert_eq!(
            actual_hashes,
            vec![string_to_signed_int("multiple words"), string_to_signed_int("two sentences"), string_to_signed_int("now three"), string_to_signed_int("four")]
        );
    }
}
