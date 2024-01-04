use std::{
    collections::{hash_map::DefaultHasher, HashSet},
    hash::{Hash, Hasher},
};

use gremlin_client::GID;

use crate::{models::NewOrthotope, up_handler, Holder, Word};
use crate::{models::NewTodo, ortho::Ortho};

pub fn handle_pair_todo_up_by_origin(todo: NewTodo, holder: &mut Holder) {
    let pair = get_pair(holder, todo.gid);
    let new_orthos = new_orthotopes_up_by_origin(holder, pair);
    let inserted_orthos = {
        let new_orthos = HashSet::from_iter(new_orthos);
        holder.insert_orthos(new_orthos)
    };
    holder.insert_todos("orthotopes", inserted_orthos);
}

pub fn handle_pair_todo_up_by_contents(todo: NewTodo, holder: &mut Holder) {
    let pair = get_pair(holder, todo.gid);
    let new_orthos = new_orthotopes_up_by_contents(holder, pair);
    let inserted_orthos = {
        let new_orthos = HashSet::from_iter(new_orthos);
        holder.insert_orthos(new_orthos)
    };
    holder.insert_todos("orthotopes", inserted_orthos);
}

pub fn handle_pair_todo_up_by_hop(todo: NewTodo, holder: &mut Holder) {
    let pair = get_pair(holder, todo.gid);
    let new_orthos = new_orthotopes_up_by_hop(holder, pair);
    let inserted_orthos = {
        let new_orthos = HashSet::from_iter(new_orthos);
        holder.insert_orthos(new_orthos)
    };
    holder.insert_todos("orthotopes", inserted_orthos);
}

pub fn handle_pair_todo_ffbb(todo: NewTodo, holder: &mut Holder) {
    let pair = get_pair(holder, todo.gid);
    let new_orthos = new_orthotopes_ffbb(holder, pair);
    let inserted_orthos = {
        let new_orthos = HashSet::from_iter(new_orthos);
        holder.insert_orthos(new_orthos)
    };
    holder.insert_todos("orthotopes", inserted_orthos);
}

pub fn handle_pair_todo_fbbf(todo: NewTodo, holder: &mut Holder) {
    let pair = get_pair(holder, todo.gid);
    let new_orthos = new_orthotopes_fbbf(holder, pair);
    let inserted_orthos = {
        let new_orthos = HashSet::from_iter(new_orthos);
        holder.insert_orthos(new_orthos)
    };
    holder.insert_todos("orthotopes", inserted_orthos);
}

pub fn handle_pair_todo_up(todo: NewTodo, holder: &mut Holder) {
    holder.insert_todos_with_gid("up_by_origin", vec![todo.gid.clone()]);
    holder.insert_todos_with_gid("up_by_hop", vec![todo.gid.clone()]);
    holder.insert_todos_with_gid("up_by_contents", vec![todo.gid]);
}

pub fn handle_pair_todo(todo: NewTodo, holder: &mut Holder) {
    holder.insert_todos_with_gid("ex_nihilo_ffbb", vec![todo.gid.clone()]);
    holder.insert_todos_with_gid("ex_nihilo_fbbf", vec![todo.gid.clone()]);
    holder.insert_todos_with_gid("pair_up", vec![todo.gid]);
}

fn single_ffbb(holder: &mut Holder, first: Word, second: Word) -> Vec<Ortho> {
    holder.ffbb(first, second)
}

fn single_fbbf(holder: &mut Holder, first: Word, second: Word) -> Vec<Ortho> {
    holder.fbbf(first, second)
}

fn new_orthotopes_up_by_origin(holder: &mut Holder, pair: (Word, Word)) -> Vec<NewOrthotope> {
    let up_orthos = up_handler::up_by_origin(holder, pair.0, pair.1);
    let up_iter = up_orthos.iter();

    let res = up_iter.map(crate::ortho_to_orthotope).collect();
    res
}

fn new_orthotopes_up_by_hop(holder: &mut Holder, pair: (Word, Word)) -> Vec<NewOrthotope> {
    let up_orthos = up_handler::up_by_hop(holder, pair.0, pair.1);
    let up_iter = up_orthos.iter();

    let res = up_iter.map(crate::ortho_to_orthotope).collect();
    res
}

fn new_orthotopes_up_by_contents(holder: &mut Holder, pair: (Word, Word)) -> Vec<NewOrthotope> {
    let up_orthos = up_handler::up_by_contents(holder, pair.0, pair.1);
    let up_iter = up_orthos.iter();

    let res = up_iter.map(crate::ortho_to_orthotope).collect();
    res
}

fn new_orthotopes_ffbb(holder: &mut Holder, pair: (Word, Word)) -> Vec<NewOrthotope> {
    let ex_nihilo_orthos = single_ffbb(holder, pair.0, pair.1);

    let nihilo_iter = ex_nihilo_orthos.iter();

    let res = nihilo_iter.map(crate::ortho_to_orthotope).collect();
    res
}

fn new_orthotopes_fbbf(holder: &mut Holder, pair: (Word, Word)) -> Vec<NewOrthotope> {
    let ex_nihilo_orthos = single_fbbf(holder, pair.0, pair.1);
    let nihilo_iter = ex_nihilo_orthos.iter();

    let res = nihilo_iter.map(crate::ortho_to_orthotope).collect();
    res
}

pub fn data_vec_to_signed_int(x: &[u8]) -> i64 {
    let mut hasher = DefaultHasher::new();
    x.hash(&mut hasher);
    hasher.finish() as i64
}

fn get_pair(holder: &Holder, pk: GID) -> (Word, Word) {
    let p = holder.get_pair(pk);

    (p.first_word, p.second_word)
}
