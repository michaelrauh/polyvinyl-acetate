use crate::{book_todo_handler, pair_todo_handler, sentence_todo_handler};
use crate::models::Todo;

pub fn handle_todo(todo: Todo) -> amiquip::Result<(), anyhow::Error> {
    match todo.domain.as_str() {
        "books" => book_todo_handler::handle_book_todo(todo),
        "sentences" => sentence_todo_handler::handle_sentence_todo(todo),
        "pairs" => pair_todo_handler::handle_pair_todo(todo),
        other => {
            panic!("getting unexpected todo with domain: {other}")
        }
    }
}
