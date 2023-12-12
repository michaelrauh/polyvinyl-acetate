use std::collections::HashSet;

use crate::{
    get_hashes_of_pairs_with_words_in, insert_orthotopes,
    models::{NewOrthotope, Todo},
    ortho::Ortho,
    over_on_ortho_found_handler, up_on_ortho_found_handler, Holder,
};

pub(crate) fn handle_ortho_todo_up(todo: crate::models::Todo, holder: &mut Holder) {
    holder.insert_todos("ortho_up_forward", vec![todo.other.into()]); // todo is it safe to use into here? // todo must a vec be made?
    holder.insert_todos("ortho_up_back", vec![todo.other.into()]); // todo is it safe to use into here? // todo must a vec be made?
}

pub(crate) fn handle_ortho_todo_up_forward(todo: crate::models::Todo, holder: &mut Holder) {
    let old_orthotope = get_orthotope(holder, todo.other);
    let new_orthos = new_orthotopes_up_forward(holder, old_orthotope);
    let inserted_orthos = insert_orthotopes(holder, HashSet::from_iter(new_orthos));
    holder.insert_todos("orthotopes", inserted_orthos);
}

pub(crate) fn handle_ortho_todo_up_back(todo: crate::models::Todo, holder: &mut Holder) {
    let old_orthotope = get_orthotope(holder, todo.other);
    let new_orthos = new_orthotopes_up_back(holder, old_orthotope);
    let inserted_orthos = insert_orthotopes(holder, HashSet::from_iter(new_orthos));
    holder.insert_todos("orthotopes", inserted_orthos);
}

pub(crate) fn handle_ortho_todo_over_forward(todo: crate::models::Todo, holder: &mut Holder) {
    let old_orthotope = get_orthotope(holder, todo.other);
    let new_orthos = new_orthotopes_over_forward(holder, old_orthotope);
    let inserted_orthos = insert_orthotopes(holder, HashSet::from_iter(new_orthos));
    holder.insert_todos("orthotopes", inserted_orthos);
}

pub(crate) fn handle_ortho_todo_over_back(todo: crate::models::Todo, holder: &mut Holder) {
    let old_orthotope = get_orthotope(holder, todo.other);
    let new_orthos = new_orthotopes_over_back(holder, old_orthotope);
    let inserted_orthos = insert_orthotopes(holder, HashSet::from_iter(new_orthos));
    holder.insert_todos("orthotopes", inserted_orthos);
}

pub(crate) fn handle_ortho_todo_over(todo: crate::models::Todo, holder: &mut Holder) {
    holder.insert_todos("ortho_over_forward", vec![todo.other.into()]); // todo is it safe to use into here? // todo must a vec be made?
    holder.insert_todos("ortho_over_back", vec![todo.other.into()]); // todo is it safe to use into here? // todo must a vec be made?
}
pub fn handle_ortho_todo(todo: Todo, holder: &mut Holder) {
    holder.insert_todos("ortho_up", vec![todo.other.into()]); // todo is it safe to use into here? // todo must a vec be made?
    holder.insert_todos("ortho_over", vec![todo.other.into()]); // todo is it safe to use into here? // todo must a vec be made?
}

fn new_orthotopes_up_forward(holder: &mut Holder, old_orthotope: Ortho) -> Vec<NewOrthotope> {
    let up_orthos = up_on_ortho_found_handler::up_forward(
        holder,
        old_orthotope,
        crate::get_ortho_by_origin_batch,
        crate::project_forward,
        get_hashes_of_pairs_with_words_in,
    );

    let orthos = up_orthos.iter();

    let res = orthos.map(crate::ortho_to_orthotope).collect();
    res
}

fn new_orthotopes_up_back(holder: &mut Holder, old_orthotope: Ortho) -> Vec<NewOrthotope> {
    let up_orthos = up_on_ortho_found_handler::up_back(
        holder,
        old_orthotope,
        crate::get_ortho_by_origin_batch,
        crate::project_backward,
        get_hashes_of_pairs_with_words_in,
    );

    let orthos = up_orthos.iter();

    let res = orthos.map(crate::ortho_to_orthotope).collect();
    res
}

fn new_orthotopes_over_forward(holder: &mut Holder, old_orthotope: Ortho) -> Vec<NewOrthotope> {
    let over_orthos: Vec<Ortho> = over_on_ortho_found_handler::over_forward(holder, old_orthotope);

    let orthos = over_orthos.iter();

    let res = orthos.map(crate::ortho_to_orthotope).collect();
    res
}

fn new_orthotopes_over_back(holder: &mut Holder, old_orthotope: Ortho) -> Vec<NewOrthotope> {
    let over_orthos: Vec<Ortho> = over_on_ortho_found_handler::over_back(holder, old_orthotope);

    let orthos = over_orthos.iter();

    let res = orthos.map(crate::ortho_to_orthotope).collect();
    res
}

fn get_orthotope(holder: &Holder, other: i32) -> Ortho {
    holder.get_orthotope(other as i64)
}
