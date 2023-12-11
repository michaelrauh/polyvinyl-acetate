pub mod models;

use maplit::hashset;

mod book_todo_handler;
pub mod ortho;
mod ortho_todo_handler;
pub mod over_on_ortho_found_handler;
mod pair_todo_handler;
pub mod phrase_ortho_handler;
pub mod phrase_todo_handler;
mod sentence_todo_handler;
mod up_handler;
mod up_helper;
mod up_on_ortho_found_handler;
pub mod worker_helper;

use crate::models::NewOrthotope;
use crate::ortho::Ortho;

use models::{Book, Pair, Sentence};
use std::collections::{HashMap, HashSet};
use std::{
    collections::hash_map::DefaultHasher,
    hash::{Hash, Hasher},
};

type Word = i32;

#[derive(Debug)]
pub struct Holder {
    // todo come back here. Must Holder store strings?
    books: HashMap<i32, Book>,
    vocabulary: HashMap<String, Word>,
    sentences: HashMap<i64, String>,
    todos: HashMap<String, HashSet<i64>>,
}

impl Holder {
    pub fn new() -> Self {
        todo!()
    }

    fn get_hashes_of_pairs_with_first_word(&self, firsts: Vec<Word>) -> HashSet<i64> {
        todo!()
    }

    fn get_vocabulary_slice_with_words(&self, firsts: HashSet<Word>) -> HashMap<Word, String> {
        todo!()
    }

    fn get_orthos_with_hops_overlapping(&self, firsts: Vec<Word>) -> Vec<Ortho> {
        todo!()
    }

    fn get_base_orthos_with_hops_overlapping(&self, firsts: Vec<Word>) -> Vec<Ortho> {
        todo!()
    }

    fn get_orthos_with_contents_overlapping(&self, firsts: Vec<Word>) -> Vec<Ortho> {
        todo!()
    }

    fn get_base_orthos_with_contents_overlapping(&self, firsts: Vec<Word>) -> Vec<Ortho> {
        todo!()
    }

    fn get_words_of_pairs_with_second_word_in(&self, firsts: HashSet<Word>) -> HashSet<(Word, Word)> {
        todo!()
    }

    fn get_words_of_pairs_with_first_word_in(&self, firsts: HashSet<Word>) -> HashSet<(Word, Word)> {
        todo!()
    }

    fn get_hashes_and_words_of_pairs_with_first_word(&self, firsts: HashSet<Word>) -> HashSet<(Word, Word, i64)> {
        todo!()
    }

    fn get_hashes_and_words_of_pairs_with_second_word(&self, firsts: HashSet<Word>) -> HashSet<(Word, Word, i64)> {
        todo!()
    }

    fn get_phrase_hash_with_phrase_head_matching(&self, firsts: HashSet<i64>) -> HashSet<i64> {
        todo!()
    }

    fn get_phrase_hash_with_phrase_tail_matching(&self, firsts: HashSet<i64>) -> HashSet<i64> {
        todo!()
    }

    fn get_phrases_matching(&self, phrases: HashSet<i64>) -> HashSet<i64> {
        todo!()
    }

    fn get_hashes_of_pairs_with_second_word(&self, seconds: Vec<Word>) -> HashSet<i64> {
        todo!()
    }

    fn get_second_words_of_pairs_with_first_word(&self, seconds: Word) -> HashSet<Word> {
        todo!()
    }

    fn get_first_words_of_pairs_with_second_word(&self, seconds: Word) -> HashSet<Word> {
        todo!()
    }

    fn get_book(&self, pk: i32) -> Book {
        self.books[&pk].clone()
    }

    fn ffbb(&self, first: Word, second: Word) -> Vec<Ortho> {
        todo!()
    }

    fn fbbf(&self, first: Word, second: Word) -> Vec<Ortho> {
        todo!()
    }

    fn insert_vocabulary(&mut self, to_insert: Vec<models::NewWords>) {
        to_insert.iter().for_each(|x| {
            self.vocabulary
                .insert(x.word.clone(), x.word_hash.try_into().unwrap()); // todo come back here and just use the word hash. Convert word to i64
        });
    }

    fn get_vocabulary(&self, words: HashSet<String>) -> HashMap<String, Word> {
        self.vocabulary
            .iter()
            .filter(|(k, _v)| words.contains(*k))
            .map(|(k, v)| (k.clone(), v.clone()))
            .collect()
    }

    fn get_pair(&self, key: i64) -> Pair {
        todo!()
    }

    fn get_phrase(&self, key: i64) -> Vec<Word> {
        todo!()
    }

    fn get_orthotope(&self, key: i64) -> Ortho {
        todo!()
    }

    fn insert_sentences(&mut self, sentences: &[models::NewSentence]) -> Vec<i64> {
        let mut new_sentences = Vec::default();
        sentences.iter().for_each(|x| {
            let k = x.sentence_hash;
            let inserted_anew = self.sentences.insert(k, x.sentence.clone()).is_none();

            if inserted_anew {
                new_sentences.push(k);
            }
        });
        new_sentences
    }

    fn insert_todos(&mut self, domain: &str, hashes: Vec<i64>) {
        let todos = self
            .todos
            .entry(domain.to_owned())
            .or_insert(HashSet::default());
        hashes.iter().for_each(|h| {
            todos.insert(*h);
        });
    }

    fn get_sentence(&self, pk: i32) -> Sentence {
        todo!()
    }

    fn insert_pairs(&self, to_insert: Vec<models::NewPair>) -> Vec<i64> {
        todo!()
    }

    fn insert_phrases(&self, to_insert: Vec<models::NewPhrase>) -> Vec<i64> {
        todo!()
    }

    fn insert_orthos(&mut self, to_insert: HashSet<NewOrthotope>) -> Vec<i64> {
        todo!()
    }

    fn get_orthos_with_origin(&mut self, to_insert: Word) -> Vec<Ortho> {
        todo!()
    }

    fn get_base_orthos_with_origin(&mut self, to_insert: Word) -> Vec<Ortho> {
        todo!()
    }

    fn get_ortho_with_origin_in(&mut self, to_insert: HashSet<Word>) -> Vec<Ortho> {
        todo!()
    }

    fn insert_book(&self, title: String, body: String) -> Book {
        todo!()
    }
}

pub fn get_hashes_of_pairs_with_words_in(
    holder: &mut Holder,
    first_words: HashSet<Word>,
    second_words: HashSet<Word>,
) -> HashSet<i64> {
    let firsts: HashSet<i64> = holder.get_hashes_of_pairs_with_first_word(Vec::from_iter(first_words));
    let seconds: HashSet<i64> = holder.get_hashes_of_pairs_with_second_word(Vec::from_iter(second_words));

    // todo consider moving into holder
    firsts.intersection(&seconds).cloned().collect()
}

pub fn get_hashes_and_words_of_pairs_with_words_in(
    holder: &Holder,
    first_words: HashSet<Word>,
    second_words: HashSet<Word>,
) -> (HashSet<Word>, HashSet<Word>, HashSet<i64>) {
    let firsts: HashSet<(Word, Word, i64)> = holder.get_hashes_and_words_of_pairs_with_first_word(first_words);
    let seconds: HashSet<(Word, Word, i64)> = holder.get_hashes_and_words_of_pairs_with_second_word(second_words);

    // todo consider moving into holder
    let domain: HashSet<(Word, Word, i64)> = firsts.intersection(&seconds).cloned().collect();
    let mut firsts = hashset! {};
    let mut seconds = hashset! {};
    let mut hashes = hashset! {};
    domain.into_iter().for_each(|(f, s, h)| {
        firsts.insert(f);
        seconds.insert(s);
        hashes.insert(h);
    });
    (firsts, seconds, hashes)
}

pub fn get_phrases_with_matching_hashes(
    holder: &Holder,
    all_phrases: HashSet<i64>,
) -> HashSet<i64> {
    holder.get_phrases_matching(all_phrases).iter()
    .cloned()
    .collect()
}

fn project_forward_batch(
    holder: &Holder,
    from: HashSet<Word>,
) -> HashSet<(Word, Word)> {
    holder.get_words_of_pairs_with_first_word_in(from)
}

fn project_backward_batch(
    holder: &Holder,
    from: HashSet<Word>,
) -> HashSet<(Word, Word)> {
    holder.get_words_of_pairs_with_second_word_in(from)
}

pub fn string_to_signed_int(t: &str) -> i64 {
    let mut hasher = DefaultHasher::new();
    t.hash(&mut hasher);
    hasher.finish() as i64
}

pub fn vec_of_words_to_big_int(v: Vec<Word>) -> i64 {
    let mut hasher = DefaultHasher::new();
    v.hash(&mut hasher);
    hasher.finish() as i64
}

pub fn vec_of_big_ints_to_big_int(v: Vec<i64>) -> i64 {
    let mut hasher = DefaultHasher::new();
    v.hash(&mut hasher);
    hasher.finish() as i64
}

pub fn string_refs_to_signed_int(l: &str, r: &str) -> i64 {
    let mut hasher = DefaultHasher::new();
    l.hash(&mut hasher);
    r.hash(&mut hasher);
    hasher.finish() as i64
}

pub fn ints_to_big_int(l: Word, r: Word) -> i64 {
    let mut hasher = DefaultHasher::new();
    l.hash(&mut hasher);
    r.hash(&mut hasher);
    hasher.finish() as i64
}

fn project_forward(
    holder: &Holder,
    from: Word,
) -> HashSet<Word> {
    holder.get_second_words_of_pairs_with_first_word(from)
}

fn project_backward(
    holder: &Holder,
    from: Word,
) -> HashSet<Word> {
    holder.get_first_words_of_pairs_with_second_word(from)
}

pub fn insert_orthotopes(holder: &mut Holder, new_orthos: HashSet<NewOrthotope>) -> Vec<i64> {
    holder.insert_orthos(new_orthos)
}

pub fn get_ortho_by_origin(
    holder: &mut Holder,
    o: Word,
) -> Vec<Ortho> {
    holder.get_orthos_with_origin(o)
}

pub fn get_base_ortho_by_origin(
    holder: &mut Holder,
    o: Word,
) -> Vec<Ortho> {
    holder.get_base_orthos_with_origin(o)
}

pub fn get_ortho_by_origin_batch(
    holder: &mut Holder,
    o: HashSet<Word>,
) -> Vec<Ortho> {
    holder.get_ortho_with_origin_in(o)
}

pub fn ortho_to_orthotope(ortho: &Ortho) -> NewOrthotope {
    let information = bincode::serialize(&ortho).expect("serialization should work");
    let origin = ortho.get_origin().to_owned();
    let hop = Vec::from_iter(ortho.get_hop());
    let contents = Vec::from_iter(ortho.get_contents());
    let info_hash = pair_todo_handler::data_vec_to_signed_int(&information);
    let base = ortho.is_base();
    NewOrthotope {
        information,
        origin,
        hop,
        contents,
        info_hash,
        base,
    }
}

fn get_ortho_by_hop(
    holder: &Holder,
    other_hop: Vec<Word>,
) -> Vec<Ortho> {
    holder.get_orthos_with_hops_overlapping(other_hop)
}

fn get_base_ortho_by_hop(
    holder: &Holder,
    other_hop: Vec<Word>,
) -> Vec<Ortho> {
    holder.get_base_orthos_with_hops_overlapping(other_hop)
}

fn get_ortho_by_contents(
    holder: &Holder,
    other_contents: Vec<Word>,
) -> Vec<Ortho> {
    holder.get_orthos_with_contents_overlapping(other_contents)
}

fn get_base_ortho_by_contents(
    holder: &Holder,
    other_contents: Vec<Word>,
) -> Vec<Ortho> {
    holder.get_base_orthos_with_contents_overlapping(other_contents)
}

pub(crate) fn phrase_exists_db_filter(
    holder: &Holder,
    left: HashSet<i64>,
    right: HashSet<i64>,
) -> HashSet<i64> {
    let firsts = holder.get_phrase_hash_with_phrase_head_matching(left);
    let seconds = holder.get_phrase_hash_with_phrase_tail_matching(right);

    firsts.intersection(&seconds).cloned().collect()
}

pub(crate) fn phrase_exists_db_filter_head(
    holder: &Holder,
    left: HashSet<i64>,
) -> HashSet<i64> {
    holder.get_phrase_hash_with_phrase_head_matching(left)
}

pub(crate) fn phrase_exists_db_filter_tail(
    holder: &Holder,
    right: HashSet<i64>,
) -> HashSet<i64> {
    holder.get_phrase_hash_with_phrase_tail_matching(right)
}

fn get_relevant_vocabulary(holder: &Holder, words: HashSet<String>) -> HashMap<String, Word> {
    holder.get_vocabulary(words)
}

fn get_relevant_vocabulary_reverse(
    holder: &Holder,
    words: HashSet<Word>,
) -> HashMap<Word, String> {
    holder.get_vocabulary_slice_with_words(words)
}
