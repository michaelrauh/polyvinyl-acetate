use diesel::r2d2::{ConnectionManager, Pool};
use diesel::PgConnection;

use crate::models::Todo;
use crate::{
    book_todo_handler, ortho_todo_handler, pair_todo_handler, phrase_todo_handler,
    sentence_todo_handler, Holder,
};

pub fn handle_todo(
    todo: Todo,
    pool: Pool<ConnectionManager<PgConnection>>,
    holder: &mut Holder,
) -> amiquip::Result<(), anyhow::Error> {
    let res = match todo.domain.as_str() {
        "books" => book_todo_handler::handle_book_todo(todo, holder),
        "sentences" => sentence_todo_handler::handle_sentence_todo(todo, pool),
        "pairs" => pair_todo_handler::handle_pair_todo(todo, pool),
        "ex_nihilo_ffbb" => pair_todo_handler::handle_pair_todo_ffbb(todo, pool),
        "ex_nihilo_fbbf" => pair_todo_handler::handle_pair_todo_fbbf(todo, pool),
        "pair_up" => pair_todo_handler::handle_pair_todo_up(todo, pool),
        "up_by_origin" => pair_todo_handler::handle_pair_todo_up_by_origin(todo, pool),
        "up_by_hop" => pair_todo_handler::handle_pair_todo_up_by_hop(todo, pool),
        "up_by_contents" => pair_todo_handler::handle_pair_todo_up_by_contents(todo, pool),
        "orthotopes" => ortho_todo_handler::handle_ortho_todo(todo, pool),
        "ortho_up" => ortho_todo_handler::handle_ortho_todo_up(todo, pool),
        "ortho_up_forward" => ortho_todo_handler::handle_ortho_todo_up_forward(todo, pool),
        "ortho_up_back" => ortho_todo_handler::handle_ortho_todo_up_back(todo, pool),
        "ortho_over" => ortho_todo_handler::handle_ortho_todo_over(todo, pool),
        "ortho_over_forward" => ortho_todo_handler::handle_ortho_todo_over_forward(todo, pool),
        "ortho_over_back" => ortho_todo_handler::handle_ortho_todo_over_back(todo, pool),
        "phrases" => phrase_todo_handler::handle_phrase_todo(todo, pool),
        "phrase_by_origin" => phrase_todo_handler::handle_phrase_todo_origin(todo, pool),
        "phrase_by_hop" => phrase_todo_handler::handle_phrase_todo_hop(todo, pool),
        "phrase_by_contents" => phrase_todo_handler::handle_phrase_todo_contents(todo, pool),
        other => {
            panic!("getting unexpected todo with domain: {other}")
        }
    };
    res
}
