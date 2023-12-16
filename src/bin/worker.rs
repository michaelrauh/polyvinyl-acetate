use std::collections::HashSet;

use polyvinyl_acetate::{get_relevant_vocabulary_reverse, models::NewTodo, worker_helper, Holder};

fn main() {
    let mut holder = Holder::new();

    let book = holder.insert_book("example".to_owned(), "a b. c d. a c. b d.".to_owned());
    get_relevant_vocabulary_reverse(&holder, HashSet::default());
    worker_helper::handle_todo(
        NewTodo {
            domain: "books".to_owned(),
            other: book.id,
        },
        &mut holder,
    )
}
