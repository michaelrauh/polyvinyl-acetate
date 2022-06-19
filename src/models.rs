use serde::Deserialize;
use serde::Serialize;

use super::schema::books;
use super::schema::orthotopes;
use super::schema::pairs;
use super::schema::phrases;
use super::schema::sentences;
use super::schema::todos;

#[derive(Insertable)]
#[table_name = "books"]
pub struct NewBook {
    pub title: String,
    pub body: String,
}

#[derive(Queryable)]
pub struct Book {
    pub id: i32,
    pub title: String,
    pub body: String,
}

#[derive(Insertable, Debug)]
#[table_name = "todos"]
pub struct NewTodo {
    pub domain: String,
    pub other: i32,
}

#[derive(Queryable, Serialize, Deserialize, Debug)]
pub struct Todo {
    pub id: i32,
    pub domain: String,
    pub other: i32,
}

#[derive(Insertable, Debug)]
#[table_name = "sentences"]
pub struct NewSentence {
    pub sentence: String,
    pub sentence_hash: i64,
}

#[derive(Queryable, Debug)]
pub struct Sentence {
    pub id: i32,
    pub sentence: String,
    pub sentence_hash: i64,
}

#[derive(Insertable, Debug)]
#[table_name = "pairs"]
pub struct NewPair {
    pub first_word: String,
    pub second_word: String,
    pub first_word_hash: i64,
    pub second_word_hash: i64,
}

#[derive(Queryable, Debug)]
pub struct Pair {
    pub id: i32,
    pub first_word: String,
    pub second_word: String,
    pub first_word_hash: i64,
    pub second_word_hash: i64,
}

#[derive(Insertable, Debug)]
#[table_name = "orthotopes"]
pub struct NewOrthotope {
    pub information: Vec<u8>,
    pub origin: String,
    pub hop: Vec<String>,
    pub contents: Vec<String>,
    pub info_hash: i64,
}

#[derive(Queryable, Debug)]
pub struct Orthotope {
    pub id: i32,
    pub information: Vec<u8>,
    pub origin: String,
    pub hop: Vec<String>,
    pub contents: Vec<String>,
    pub info_hash: i64,
}

#[derive(Insertable, Debug)]
#[table_name = "phrases"]
pub struct NewPhrase {
    pub words: Vec<String>,
    pub words_hash: i64,
}

#[derive(Queryable, Debug)]
pub struct Phrase {
    pub id: i32,
    pub words: Vec<String>,
    pub words_hash: i64,
}
