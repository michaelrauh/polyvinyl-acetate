use std::{collections::HashSet, fs::{self}};

use polyvinyl_acetate::{get_relevant_vocabulary_reverse, worker_helper, Holder};

fn main() {
    let mut holder = Holder::new();

    if !holder.unprocessed_todos_exist() {
        let f = fs::read_to_string("input.txt").unwrap();

        let book = holder.insert_book("example".to_owned(), f);
        holder.insert_todos("books", vec![book]);
    }

    let mut i = 0;
    loop {
        i += 1;
        if i % 10 == 0 {
            dbg!();
            dbg!(i);
            holder.get_stats();
            holder.save_todos();
        }
        let next_todo = holder.get_next_todo();

        if next_todo.is_none() {
            break;
        } else {
            worker_helper::handle_todo(next_todo.clone().unwrap(), &mut holder);
            holder.complete_todo(next_todo.unwrap());
        }
    }

    get_relevant_vocabulary_reverse(&holder, HashSet::default());
}