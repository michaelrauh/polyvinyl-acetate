use std::collections::HashSet;

use crate::models::{NewBook, NewSentence, NewTodo, NewWords};
use crate::{string_to_signed_int, Holder};

use itertools::Itertools;

pub fn handle_book_todo(todo: NewTodo, holder: &mut Holder) {
    let book = holder.get_book(todo.other);
    let new_vocabulary = split_book_to_words(&book);
    insert_vocabulary(holder, &new_vocabulary);
    let new_sentences = split_book_to_sentences(&book);
    let sentence_hashes = insert_sentences(holder, &new_sentences);
    holder.insert_todos("sentences", sentence_hashes);
}

fn insert_vocabulary(holder: &mut Holder, vocabulary: &HashSet<String>) {
    let to_insert: Vec<NewWords> = vocabulary
        .iter()
        .map(|s| NewWords {
            word_hash: string_to_signed_int(s),
            word: s.clone(),
        })
        .collect();

    holder.insert_vocabulary(to_insert);
}

fn split_book_to_words(book: &NewBook) -> HashSet<String> {
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

fn insert_sentences(holder: &mut Holder, sentences: &[NewSentence]) -> Vec<i64> {
    holder.insert_sentences(sentences)
}

pub fn split_book_to_sentences(book: &NewBook) -> Vec<NewSentence> {
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
    use crate::models::NewBook;
    use crate::string_to_signed_int;

    #[test]
    fn it_splits_books_to_sentences() {
        let book = NewBook {
            title: "title".to_owned(),
            body: "Multiple words.. \n\tTwo sentences! Now,:- three; Four.".to_owned(),
        };
        let actual = split_book_to_sentences(&book);
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
