use crate::models::Todo;

use crate::{
    book_todo_handler, ortho_todo_handler, pair_todo_handler, phrase_todo_handler,
    sentence_todo_handler, Holder,
};

pub fn handle_todo(todo: Todo, holder: &mut Holder) {
    let res: () = match todo.domain.as_str() {
        "books" => book_todo_handler::handle_book_todo(todo, holder),
        "sentences" => sentence_todo_handler::handle_sentence_todo(todo, holder),
        "pairs" => pair_todo_handler::handle_pair_todo(todo, holder),
        "ex_nihilo_ffbb" => pair_todo_handler::handle_pair_todo_ffbb(todo, holder),
        "ex_nihilo_fbbf" => pair_todo_handler::handle_pair_todo_fbbf(todo, holder),
        "pair_up" => pair_todo_handler::handle_pair_todo_up(todo, holder),
        "up_by_origin" => pair_todo_handler::handle_pair_todo_up_by_origin(todo, holder),
        "up_by_hop" => pair_todo_handler::handle_pair_todo_up_by_hop(todo, holder),
        "up_by_contents" => pair_todo_handler::handle_pair_todo_up_by_contents(todo, holder),
        "orthotopes" => ortho_todo_handler::handle_ortho_todo(todo, holder),
        "ortho_up" => ortho_todo_handler::handle_ortho_todo_up(todo, holder),
        "ortho_up_forward" => ortho_todo_handler::handle_ortho_todo_up_forward(todo, holder),
        "ortho_up_back" => ortho_todo_handler::handle_ortho_todo_up_back(todo, holder),
        "ortho_over" => ortho_todo_handler::handle_ortho_todo_over(todo, holder),
        "ortho_over_forward" => ortho_todo_handler::handle_ortho_todo_over_forward(todo, holder),
        "ortho_over_back" => ortho_todo_handler::handle_ortho_todo_over_back(todo, holder),
        "phrases" => phrase_todo_handler::handle_phrase_todo(todo, holder),
        "phrase_by_origin" => phrase_todo_handler::handle_phrase_todo_origin(todo, holder),
        "phrase_by_hop" => phrase_todo_handler::handle_phrase_todo_hop(todo, holder),
        "phrase_by_contents" => phrase_todo_handler::handle_phrase_todo_contents(todo, holder),
        _ => panic!(),
    };
    res
}
