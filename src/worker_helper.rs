use crate::models::Todo;
use crate::{book_todo_handler, pair_todo_handler, sentence_todo_handler};

pub fn handle_todo(todo: Todo) -> amiquip::Result<(), anyhow::Error> {
    match todo.domain.as_str() {
        "books" => book_todo_handler::handle_book_todo(todo),
        "sentences" => sentence_todo_handler::handle_sentence_todo(todo),
        "pairs" => pair_todo_handler::handle_pair_todo(todo),
        "orthotopes" => {
            println!("dropping todo: {:?}", todo);
            Ok(())
        }
        other => {
            panic!("getting unexpected todo with domain: {other}")
        }
    }
}
