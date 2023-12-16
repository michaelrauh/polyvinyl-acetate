use std::collections::HashSet;

use polyvinyl_acetate::{get_relevant_vocabulary_reverse, worker_helper, Holder};

fn main() {
    let mut holder = Holder::default();

    let book = holder.insert_book("example".to_owned(), "a b. c d. a c. b d.".to_owned());
    holder.insert_todos("books", vec![book.id]);

    loop {
        let next_todo = holder.get_next_todo();

        if next_todo.is_none() {
            break;
        }

        worker_helper::handle_todo(next_todo.unwrap(), &mut holder)
    }

    get_relevant_vocabulary_reverse(&holder, HashSet::default());
}
