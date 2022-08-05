use std::collections::HashSet;

use crate::models::{NewSentence, NewWords, Sentence, Todo};
use crate::schema::books::{id, table as books};
use crate::schema::words::{self};
use crate::{
    create_todo_entry, establish_connection_safe, schema, sentences, string_to_signed_int, Book,
    NewTodo,
};

use diesel::{ExpressionMethods, PgConnection, QueryDsl, RunQueryDsl};
use itertools::Itertools;

pub fn handle_book_todo(todo: Todo) -> Result<(), anyhow::Error> {
    let conn = establish_connection_safe()?;
    conn.build_transaction().serializable().run(|| {
        let book = get_book(&conn, todo.other)?;
        let new_vocabulary = split_book_to_words(&book);
        insert_vocabulary(&conn, &new_vocabulary)?;
        let new_sentences = split_book_to_sentences(book);
        let sentences = insert_sentences(&conn, &new_sentences)?;
        let todos: Vec<NewTodo> = sentences
            .iter()
            .map(|s| NewTodo {
                domain: "sentences".to_owned(),
                other: s.id,
            })
            .collect();
        create_todo_entry(&conn, todos)?;
        Ok(())
    })
}

fn insert_vocabulary(
    conn: &PgConnection,
    vocabulary: &HashSet<String>,
) -> Result<usize, diesel::result::Error> {
    let to_insert: Vec<NewWords> = vocabulary
        .iter()
        .map(|s| NewWords {
            word_hash: string_to_signed_int(s),
            word: s.clone(),
        })
        .collect();
    diesel::insert_into(words::table)
        .values(to_insert)
        .on_conflict_do_nothing()
        .execute(conn)
}

fn split_book_to_words(book: &Book) -> HashSet<String> {
    book.body
        .split_terminator(&['.', '!', '?', ';'])
        .filter(|x| !x.is_empty())
        .map(|x| x.trim())
        .flat_map(|sentence| {
            sentence
                .split_ascii_whitespace()
                .map(|s| {
                    s.chars()
                        .filter(|c| c.is_alphabetic())
                        .collect::<String>()
                        .to_lowercase()
                })
                .collect::<Vec<String>>()
        })
        .collect()
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
    let book: Book = books
        .filter(id.eq(pk))
        .select(schema::books::all_columns)
        .first(conn)?;

    Ok(book)
}

pub fn split_book_to_sentences(book: Book) -> Vec<NewSentence> {
    book.body
        .split_terminator(&['.', '!', '?', ';'])
        .filter(|x| !x.is_empty())
        .map(|x| x.trim())
        .map(|sentence| {
            sentence
                .split_ascii_whitespace()
                .map(|s| {
                    s.chars()
                        .filter(|c| c.is_alphabetic())
                        .collect::<String>()
                        .to_lowercase()
                })
                .join(" ")
        })
        .map(|t| NewSentence {
            sentence: t.clone(),
            sentence_hash: string_to_signed_int(&t),
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use crate::book_todo_handler::split_book_to_sentences;
    use crate::models::Book;
    use crate::string_to_signed_int;

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
            vec![
                string_to_signed_int("multiple words"),
                string_to_signed_int("two sentences"),
                string_to_signed_int("now three"),
                string_to_signed_int("four")
            ]
        );
    }
}
