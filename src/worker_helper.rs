use crate::models::Todo;
use crate::{
    book_todo_handler, ortho_todo_handler, pair_todo_handler, phrase_todo_handler,
    sentence_todo_handler,
};

#[tracing::instrument(level = "info")]
pub fn handle_todo(todo: Todo) -> amiquip::Result<(), anyhow::Error> {
    tracing::info!("in this one!!!!");
    let res = match todo.domain.as_str() {
        "books" => book_todo_handler::handle_book_todo(todo),
        "sentences" => sentence_todo_handler::handle_sentence_todo(todo),
        "pairs" => pair_todo_handler::handle_pair_todo(todo),
        "ex_nihilo_ffbb" => pair_todo_handler::handle_pair_todo_ffbb(todo),
        "ex_nihilo_fbbf" => pair_todo_handler::handle_pair_todo_fbbf(todo),
        "pair_up" => pair_todo_handler::handle_pair_todo_up(todo),
        "up_by_origin" => pair_todo_handler::handle_pair_todo_up_by_origin(todo),
        "up_by_hop" => pair_todo_handler::handle_pair_todo_up_by_hop(todo),
        "up_by_contents" => pair_todo_handler::handle_pair_todo_up_by_contents(todo),
        "orthotopes" => ortho_todo_handler::handle_ortho_todo(todo),
        "ortho_up" => ortho_todo_handler::handle_ortho_todo_up(todo),
        "ortho_up_forward" => ortho_todo_handler::handle_ortho_todo_up_forward(todo),
        "ortho_up_back" => ortho_todo_handler::handle_ortho_todo_up_back(todo),
        "ortho_over" => ortho_todo_handler::handle_ortho_todo_over(todo),
        "ortho_over_forward" => ortho_todo_handler::handle_ortho_todo_over_forward(todo),
        "ortho_over_back" => ortho_todo_handler::handle_ortho_todo_over_back(todo),
        "phrases" => phrase_todo_handler::handle_phrase_todo(todo),
        "phrase_by_origin" => phrase_todo_handler::handle_phrase_todo_origin(todo),
        "phrase_by_hop" => phrase_todo_handler::handle_phrase_todo_hop(todo),
        "phrase_by_contents" => phrase_todo_handler::handle_phrase_todo_contents(todo),
        other => {
            panic!("getting unexpected todo with domain: {other}")
        }
    };
    res
}
