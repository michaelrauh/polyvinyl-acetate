use serde::Deserialize;
use serde::Serialize;

use crate::ortho::Ortho;
use crate::Word;

pub struct NewBook {
    pub title: String,
    pub body: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Book {
    pub id: i64,
    pub title: String,
    pub body: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct NewTodo {
    pub domain: String,
    pub other: i64,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Todo {
    pub id: i32,
    pub domain: String,
    pub other: i32,
}

#[derive(Debug)]
pub struct NewSentence {
    pub sentence: String,
    pub sentence_hash: i64,
}

#[derive(Debug)]
pub struct Sentence {
    pub id: i32,
    pub sentence: String,
    pub sentence_hash: i64,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct NewPair {
    pub first_word: Word,
    pub second_word: Word,
    pub pair_hash: i64,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Pair {
    pub id: i32,
    pub first_word: Word,
    pub second_word: Word,
    pub pair_hash: i64,
}

#[derive(Debug, PartialEq, Eq, Hash, Clone)]
pub struct NewOrthotope {
    pub information: Ortho,
    pub origin: Word,
    pub hop: Vec<Word>,
    pub contents: Vec<Word>,
    pub base: bool,
    pub info_hash: i64,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Orthotope {
    pub id: i32,
    pub information: Ortho,
    pub origin: Word,
    pub hop: Vec<Word>,
    pub contents: Vec<Word>,
    pub base: bool,
    pub info_hash: i64,
}

#[derive(Debug, PartialEq, Eq, Hash, Clone)]
pub struct NewWords {
    pub word: String,
    pub word_hash: i64,
}

#[derive(Debug)]
pub struct Words {
    pub id: i32,
    pub word: String,
    pub word_hash: i64,
}

#[derive(Debug)]
pub struct NewPhrase {
    pub words: Vec<Word>,
    pub phrase_head: i64,
    pub phrase_tail: i64,
    pub words_hash: i64,
}

#[derive(Debug)]
pub struct Phrase {
    pub id: i32,
    pub words: Vec<Word>,
    pub phrase_head: i64,
    pub phrase_tail: i64,
    pub words_hash: i64,
}

#[derive(Debug)]
pub struct ExNihilo {
    pub first_word: Word,
    pub second_word: Word,
}
