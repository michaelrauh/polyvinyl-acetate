use std::fs::{self, DirEntry, read_dir};

use itertools::Itertools;
use polyvinyl_acetate::{worker_helper, Holder};
use rand::{Rng, distributions::Alphanumeric, thread_rng};

fn main() {
    let folder_name: String = thread_rng()
        .sample_iter(&Alphanumeric)
        .take(30)
        .map(char::from)
        .collect();

    let mut db_name = folder_name.clone();

    db_name.push_str(".redb");
    let mut holder = Holder::new(db_name.clone());

    if merge_needed() {
        let (lhs, rhs) = move_dbs(folder_name);
        let todos = holder.get_merged_todos(lhs.clone(), rhs.clone());
        holder.write_todos_back(todos);
        holder.rehydrate_todos();
        holder.merge_dbs(lhs, rhs);
        remove_old_dbs();
    } else {
        if !holder.unprocessed_todos_exist() {
            let f = fs::read_to_string("input.txt").unwrap();
    
            let book = holder.insert_book("example".to_owned(), f);
            holder.insert_todos("books", vec![book]);
        }
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

    let mut target: String = "completed/".to_string();
    target.push_str(&db_name);
    fs::rename(db_name, target).unwrap();
}

fn remove_old_dbs() {
    // TODO implement
}

fn move_dbs(folder_name: String) -> (String, String) {
    let paths = read_dir("completed").unwrap();
    let ps: (DirEntry, DirEntry) = paths.take(2).map(|x| x.unwrap()).collect_tuple().unwrap();
    let binding = ps.0.file_name();
    let file_one =binding.to_str().unwrap();
    let binding = ps.1.file_name();
    let file_two =binding.to_str().unwrap();

    fs::create_dir(folder_name.clone()).unwrap();
    let mut old_lhs: String = "completed".to_string();
    old_lhs.push_str(file_one);
    let mut old_rhs: String = "completed".to_string();
    old_rhs.push_str(file_two);
    let mut new_lhs: String = folder_name.clone();
    new_lhs.push_str("lhs.redb");
    let mut new_rhs: String = folder_name.clone();
    new_rhs.push_str("rhs.redb");
    fs::rename(old_lhs, new_lhs.clone()).unwrap();
    fs::rename(old_rhs, new_rhs.clone()).unwrap();

    (new_lhs, new_rhs)
}

fn merge_needed() -> bool {
    let paths = read_dir("completed").unwrap();
    paths.count() > 1
}