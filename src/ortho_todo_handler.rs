use std::collections::HashSet;

use crate::{
    models::{NewOrthotope, NewTodo},
    ortho::Ortho,
    over_on_ortho_found_handler, up_on_ortho_found_handler, Holder,
};

pub(crate) fn handle_ortho_todo_up(todo: crate::models::NewTodo, holder: &mut Holder) {
    holder.insert_todos("ortho_up_forward", vec![todo.other]);
    holder.insert_todos("ortho_up_back", vec![todo.other]);
}

pub(crate) fn handle_ortho_todo_up_forward(todo: crate::models::NewTodo, holder: &mut Holder) {
    let old_orthotope = get_orthotope(holder, todo.other);
    let new_orthos = new_orthotopes_up_forward(holder, old_orthotope);
    let inserted_orthos = {
        let new_orthos = HashSet::from_iter(new_orthos);
        holder.insert_orthos(new_orthos)
    };
    holder.insert_todos("orthotopes", inserted_orthos);
}

pub(crate) fn handle_ortho_todo_up_back(todo: crate::models::NewTodo, holder: &mut Holder) {
    let old_orthotope = get_orthotope(holder, todo.other);
    let new_orthos = new_orthotopes_up_back(holder, old_orthotope);
    let inserted_orthos = {
        let new_orthos = HashSet::from_iter(new_orthos);
        holder.insert_orthos(new_orthos)
    };
    holder.insert_todos("orthotopes", inserted_orthos);
}

pub(crate) fn handle_ortho_todo_over_forward(todo: crate::models::NewTodo, holder: &mut Holder) {
    let old_orthotope = get_orthotope(holder, todo.other);
    let new_orthos = new_orthotopes_over_forward(holder, old_orthotope);
    let inserted_orthos = {
        let new_orthos = HashSet::from_iter(new_orthos);
        holder.insert_orthos(new_orthos)
    };
    holder.insert_todos("orthotopes", inserted_orthos);
}

pub(crate) fn handle_ortho_todo_over_back(todo: crate::models::NewTodo, holder: &mut Holder) {
    let old_orthotope = get_orthotope(holder, todo.other);
    let new_orthos = new_orthotopes_over_back(holder, old_orthotope);
    let inserted_orthos = {
        let new_orthos = HashSet::from_iter(new_orthos);
        holder.insert_orthos(new_orthos)
    };
    holder.insert_todos("orthotopes", inserted_orthos);
}

pub(crate) fn handle_ortho_todo_over(todo: crate::models::NewTodo, holder: &mut Holder) {
    holder.insert_todos("ortho_over_forward", vec![todo.other]);
    holder.insert_todos("ortho_over_back", vec![todo.other]);
}
pub fn handle_ortho_todo(todo: NewTodo, holder: &mut Holder) {
    holder.insert_todos("ortho_up", vec![todo.other]);
    holder.insert_todos("ortho_over", vec![todo.other]);
}

fn new_orthotopes_up_forward(holder: &mut Holder, old_orthotope: Ortho) -> Vec<NewOrthotope> {
    let up_orthos = up_on_ortho_found_handler::up_forward(
        holder,
        old_orthotope,
    );

    let orthos = up_orthos.iter();

    let res = orthos.map(crate::ortho_to_orthotope).collect();
    res
}

fn new_orthotopes_up_back(holder: &mut Holder, old_orthotope: Ortho) -> Vec<NewOrthotope> {
    let up_orthos = up_on_ortho_found_handler::up_back(
        holder,
        old_orthotope,
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

fn get_orthotope(holder: &Holder, other: i64) -> Ortho {
    holder.get_orthotope(other)
}
