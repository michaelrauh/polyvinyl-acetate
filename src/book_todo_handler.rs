use crate::models::{NewSentence, Sentence, Todo};
use crate::schema::books::{id, table as books};
use crate::{
    create_todo_entry, establish_connection, schema, sentences, string_to_signed_int, Book, NewTodo,
};
use diesel::{ExpressionMethods, PgConnection, QueryDsl, RunQueryDsl};

pub fn handle_book_todo(todo: Todo) -> Result<(), anyhow::Error> {
    let conn = establish_connection();
    conn.build_transaction().serializable().run(|| {
        let book = get_book(&conn, todo.other)?;
        let new_sentences = split_book_to_sentences(book);
        let sentences = insert_sentences(&conn, &new_sentences)?;
        let todos = sentences
            .iter()
            .map(|s| NewTodo {
                domain: "sentences".to_owned(),
                other: s.id,
            })
            .collect();
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
    let book: Book = books
        .filter(id.eq(pk))
        .select(schema::books::all_columns)
        .first(conn)?;

    Ok(book)
}

pub fn split_book_to_sentences(book: Book) -> Vec<NewSentence> {
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
                .to_lowercase()
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
