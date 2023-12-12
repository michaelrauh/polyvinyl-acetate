use std::collections::HashSet;

use crate::models::Todo;
use crate::ortho_to_orthotope;
use crate::phrase_ortho_handler;
use crate::Holder;
use crate::Word;

use crate::{insert_orthotopes, models::NewOrthotope};

pub(crate) fn handle_phrase_todo_origin(todo: crate::models::Todo, holder: &mut Holder) {
    let phrase = get_phrase(holder, todo.other);
    let new_orthos = new_orthotopes_by_origin(holder, phrase);
    let inserted_orthos = insert_orthotopes(holder, HashSet::from_iter(new_orthos));
    holder.insert_todos("orthotopes", inserted_orthos);
}

pub(crate) fn handle_phrase_todo_hop(todo: crate::models::Todo, holder: &mut Holder) {
    let phrase = get_phrase(holder, todo.other);
    let new_orthos = new_orthotopes_by_hop(holder, phrase);
    let inserted_orthos = insert_orthotopes(holder, HashSet::from_iter(new_orthos));
    holder.insert_todos("orthotopes", inserted_orthos);
}

pub(crate) fn handle_phrase_todo_contents(todo: crate::models::Todo, holder: &mut Holder) {
    let phrase = get_phrase(holder, todo.other);
    let new_orthos = new_orthotopes_by_contents(holder, phrase);
    let inserted_orthos = insert_orthotopes(holder, HashSet::from_iter(new_orthos));
    holder.insert_todos("orthotopes", inserted_orthos);
}

pub fn handle_phrase_todo(todo: Todo, holder: &mut Holder) {
    holder.insert_todos("phrase_by_origin", vec![todo.other.into()]); // todo is it safe to use into here? // todo must a vec be made?
    holder.insert_todos("phrase_by_hop", vec![todo.other.into()]); // todo is it safe to use into here? // todo must a vec be made?
    holder.insert_todos("phrase_by_contents", vec![todo.other.into()]); // todo is it safe to use into here? // todo must a vec be made?
}

fn new_orthotopes_by_origin(holder: &mut Holder, phrase: Vec<Word>) -> Vec<NewOrthotope> {
    let orthos = phrase_ortho_handler::over_by_origin(holder, phrase);

    let res = orthos.iter().map(ortho_to_orthotope).collect();
    res
}

fn new_orthotopes_by_hop(holder: &mut Holder, phrase: Vec<Word>) -> Vec<NewOrthotope> {
    let orthos = phrase_ortho_handler::over_by_hop(holder, phrase);

    let res = orthos.iter().map(ortho_to_orthotope).collect();
    res
}

fn new_orthotopes_by_contents(holder: &mut Holder, phrase: Vec<Word>) -> Vec<NewOrthotope> {
    let orthos = phrase_ortho_handler::over_by_contents(holder, phrase);

    let res = orthos.iter().map(ortho_to_orthotope).collect();
    res
}

fn get_phrase(holder: &mut Holder, pk: i32) -> Vec<Word> {
    holder.get_phrase(pk.into())
}
