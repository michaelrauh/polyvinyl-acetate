pub mod models;

use gremlin_client::structure::P;
use gremlin_client::GID;
use gremlin_client::{process::traversal::traversal, GremlinClient};
use itertools::Itertools;

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

use models::{Book, NewPair, NewTodo};
use std::collections::{HashMap, HashSet};
use std::{
    collections::hash_map::DefaultHasher,
    hash::{Hash, Hasher},
};

type Word = i64;

pub struct Holder {
    todos: Vec<NewTodo>,
    phrases_by_head: HashMap<i64, HashSet<i64>>,
    phrases_by_tail: HashMap<i64, HashSet<i64>>,
    phrases_by_hash: HashMap<i64, Vec<Word>>,
    orthos_by_hash: HashMap<i64, Ortho>,
    orthos_by_hop: HashMap<Word, HashSet<NewOrthotope>>,
    orthos_by_contents: HashMap<Word, HashSet<NewOrthotope>>,
    orthos_by_origin: HashMap<Word, HashSet<Ortho>>,
    g: gremlin_client::process::traversal::GraphTraversalSource<
        gremlin_client::process::traversal::SyncTerminator,
    >,
}

impl Holder {
    pub fn new() -> Self {
        Holder {
            todos: Vec::default(),
            phrases_by_head: HashMap::default(),
            phrases_by_tail: HashMap::default(),
            phrases_by_hash: HashMap::default(),
            orthos_by_hash: HashMap::default(),
            orthos_by_hop: HashMap::default(),
            orthos_by_contents: HashMap::default(),
            orthos_by_origin: HashMap::default(),
            g: traversal().with_remote(GremlinClient::connect("localhost").unwrap()),
        }
    }

    pub fn get_stats(&self) {
        dbg!(&self.todos.len());
        dbg!(&self.orthos_by_hash.len());
    }

    fn get_hashes_of_pairs_with_first_word(&self, firsts: Vec<Word>) -> HashSet<i64> {
        self.g
            .v(firsts)
            .out_e("pair")
            .properties("hash")
            .to_list()
            .unwrap()
            .iter()
            .map(|p| p.get::<i64>().unwrap().to_owned())
            .collect()
    }

    pub fn get_vocabulary_slice_with_words(&self, firsts: HashSet<Word>) -> HashMap<Word, String> {
        self.g
            .v(())
            .has_label("word")
            .has(("id", P::within(firsts.into_iter().collect_vec())))
            .to_list()
            .unwrap()
            .iter()
            .map(|v| {
                let val = v
                    .property("value")
                    .unwrap()
                    .get::<String>()
                    .unwrap()
                    .to_owned();
                let w = v.id().get::<i64>().unwrap().to_owned();
                (w, val)
            })
            .collect::<HashMap<Word, String>>()
    }

    fn get_orthos_with_hops_overlapping(&self, hop: Vec<Word>) -> Vec<Ortho> {
        hop.iter()
            .flat_map(|h| {
                self.orthos_by_hop
                    .get(h)
                    .unwrap_or(&HashSet::default())
                    .iter()
                    .map(|o| o.information.clone())
                    .collect_vec()
            })
            .collect()
    }

    fn get_base_orthos_with_hops_overlapping(&self, hop: Vec<Word>) -> Vec<Ortho> {
        hop.iter()
            .flat_map(|h| {
                self.orthos_by_hop
                    .get(h)
                    .unwrap_or(&HashSet::default())
                    .iter()
                    .filter(|o| o.base)
                    .map(|o| o.information.clone())
                    .collect_vec()
            })
            .collect()
    }

    fn get_orthos_with_contents_overlapping(&self, other_contents: Vec<Word>) -> Vec<Ortho> {
        other_contents
            .iter()
            .flat_map(|c| {
                self.orthos_by_contents
                    .get(c)
                    .unwrap_or(&HashSet::default())
                    .iter()
                    .map(|o| o.information.clone())
                    .collect_vec()
            })
            .collect()
    }

    fn get_base_orthos_with_contents_overlapping(&self, other_contents: Vec<Word>) -> Vec<Ortho> {
        other_contents
            .iter()
            .flat_map(|c| {
                self.orthos_by_contents
                    .get(c)
                    .unwrap_or(&HashSet::default())
                    .iter()
                    .filter(|o| o.base)
                    .map(|o| o.information.clone())
                    .collect_vec()
            })
            .collect()
    }

    fn get_words_of_pairs_with_second_word_in(&self, from: HashSet<Word>) -> HashSet<(Word, Word)> {
        let edges = self
            .g
            .v(from.into_iter().collect_vec())
            .in_e("pair")
            .to_list()
            .unwrap();

        edges
            .iter()
            .map(|e| {
                let v1 = e.clone().out_v().id().get::<i64>().unwrap().to_owned();
                let v2 = e.clone().in_v().id().get::<i64>().unwrap().to_owned();
                (v1, v2)
            })
            .collect()
    }

    fn get_words_of_pairs_with_first_word_in(&self, from: HashSet<Word>) -> HashSet<(Word, Word)> {
        let edges = self
            .g
            .v(from.into_iter().collect_vec())
            .out_e("pair")
            .to_list()
            .unwrap();

        edges
            .iter()
            .map(|e| {
                let v1 = e.clone().out_v().id().get::<i64>().unwrap().to_owned();
                let v2 = e.clone().in_v().id().get::<i64>().unwrap().to_owned();
                (v1, v2)
            })
            .collect()
    }

    fn get_hashes_and_words_of_pairs_with_first_word(
        &self,
        from: HashSet<Word>,
    ) -> HashSet<(Word, Word, i64)> {
        let edges = self
            .g
            .v(from.into_iter().collect_vec())
            .out_e("pair")
            .to_list()
            .unwrap();

        edges
            .iter()
            .map(|e| {
                let id = e.id().get::<i64>().unwrap().to_owned();
                let h = self
                    .g
                    .e(id)
                    .properties("hash")
                    .next()
                    .unwrap()
                    .unwrap()
                    .get::<i64>()
                    .unwrap()
                    .clone();

                let v1 = e.clone().out_v().id().get::<i64>().unwrap().to_owned();
                let v2 = e.clone().in_v().id().get::<i64>().unwrap().to_owned();
                (v1, v2, h)
            })
            .collect()
    }

    fn get_hashes_and_words_of_pairs_with_second_word(
        &self,
        from: HashSet<Word>,
    ) -> HashSet<(Word, Word, i64)> {
        let edges = self
            .g
            .v(from.into_iter().collect_vec())
            .in_e("pair")
            .to_list()
            .unwrap();

        edges
            .iter()
            .map(|e| {
                let id = e.id().get::<i64>().unwrap().to_owned();
                let h = self
                    .g
                    .e(id)
                    .properties("hash")
                    .next()
                    .unwrap()
                    .unwrap()
                    .get::<i64>()
                    .unwrap()
                    .clone();

                let v1 = e.clone().out_v().id().get::<i64>().unwrap().to_owned();
                let v2 = e.clone().in_v().id().get::<i64>().unwrap().to_owned();
                (v1, v2, h)
            })
            .collect()
    }

    fn get_phrase_hash_with_phrase_head_matching(&self, left: HashSet<i64>) -> HashSet<i64> {
        left.iter()
            .flat_map(|f| {
                self.phrases_by_head
                    .get(f)
                    .unwrap_or(&HashSet::default())
                    .iter()
                    .copied()
                    .collect_vec()
            })
            .collect()
    }

    fn get_phrase_hash_with_phrase_tail_matching(&self, left: HashSet<i64>) -> HashSet<i64> {
        left.iter()
            .flat_map(|f| {
                self.phrases_by_tail
                    .get(f)
                    .unwrap_or(&HashSet::default())
                    .iter()
                    .copied()
                    .collect_vec()
            })
            .collect()
    }

    fn get_phrases_matching(&self, phrases: HashSet<i64>) -> HashSet<i64> {
        phrases
            .iter()
            .flat_map(|f| {
                self.phrases_by_hash
                    .get(f)
                    .iter()
                    .map(|p| vec_of_words_to_big_int(p.to_vec()))
                    .collect_vec()
            })
            .collect()
    }

    fn get_hashes_of_pairs_with_second_word(&self, seconds: Vec<Word>) -> HashSet<i64> {
        self.g
            .v(seconds)
            .in_e("pair")
            .properties("hash")
            .to_list()
            .unwrap()
            .iter()
            .map(|p| p.get::<i64>().unwrap().to_owned())
            .collect()
    }

    fn get_second_words_of_pairs_with_first_word(&self, first: Word) -> HashSet<Word> {
        self.g
            .v(first)
            .out("pair")
            .to_list()
            .unwrap()
            .iter()
            .map(|v| v.id().get::<i64>().unwrap().to_owned())
            .collect()
    }

    fn get_first_words_of_pairs_with_second_word(&self, second: Word) -> HashSet<Word> {
        self.g
            .v(second)
            .in_("pair")
            .to_list()
            .unwrap()
            .iter()
            .map(|v| v.id().get::<i64>().unwrap().to_owned())
            .collect()
    }

    fn get_book(&self, pk: GID) -> Book {
        let b = self.g.v(pk.clone());
        let body: String = b
            .clone()
            .values("body")
            .next()
            .unwrap()
            .unwrap()
            .get::<String>()
            .unwrap()
            .to_string();
        let title = b
            .values("title")
            .next()
            .unwrap()
            .unwrap()
            .get::<String>()
            .unwrap()
            .to_string();

        Book {
            id: pk,
            title,
            body,
        }
    }

    fn ffbb(&self, a: Word, b: Word) -> Vec<Ortho> {
        // a b
        // c d

        // a -> b
        // b -> d
        // d <- c
        // c <- a

        let ans: Vec<Ortho> = self
            .get_second_words_of_pairs_with_first_word(b)
            .into_iter()
            .flat_map(|d| {
                self.get_first_words_of_pairs_with_second_word(d)
                    .into_iter()
                    .filter(|c| b != *c)
                    .flat_map(move |c| {
                        self.get_first_words_of_pairs_with_second_word(c)
                            .into_iter()
                            .filter(|a_prime| a_prime == &a)
                            .map(|_| (a, b, c, d))
                            .collect_vec()
                    })
            })
            .map(|(a, b, c, d)| Ortho::new(a, b, c, d))
            .collect();

        ans
    }

    fn fbbf(&self, b: Word, d: Word) -> Vec<Ortho> {
        // a b
        // c d

        // b -> d
        // d <- c
        // c <- a
        // a -> b
        let ans: Vec<Ortho> = self
            .get_first_words_of_pairs_with_second_word(d)
            .into_iter()
            .flat_map(|c| {
                self.get_first_words_of_pairs_with_second_word(c)
                    .into_iter()
                    .flat_map(move |a| {
                        self.get_second_words_of_pairs_with_first_word(a)
                            .into_iter()
                            .filter(|b_prime| b_prime == &b)
                            .map(|_| (a, b, c, d))
                            .collect_vec()
                    })
            })
            .filter(|(_a, b, c, _d)| b != c)
            .map(|(a, b, c, d)| Ortho::new(a, b, c, d))
            .collect();
        ans
    }

    fn insert_vocabulary(&mut self, to_insert: Vec<models::NewWords>) {
        let existing = self
            .g
            .v(())
            .has_label("word")
            .has((
                "value",
                P::within(to_insert.iter().map(|x| x.word.clone()).collect_vec()),
            ))
            .values("value")
            .to_list()
            .unwrap()
            .iter()
            .map(|v| v.get::<String>().unwrap().to_owned())
            .collect::<HashSet<String>>();

        to_insert.iter().for_each(|word| {
            let w = &word.word;
            if !existing.contains(w) {
                self.g
                    .add_v("word")
                    .property("value", w.clone())
                    .next()
                    .unwrap();
            }
        });
    }

    fn get_vocabulary(&self, words: HashSet<String>) -> HashMap<String, Word> {
        let ans = self
            .g
            .v(())
            .has_label("word")
            .has(("value", P::within(words.into_iter().collect_vec())))
            .to_list()
            .unwrap()
            .iter()
            .map(|v| {
                let val = v
                    .property("value")
                    .unwrap()
                    .get::<String>()
                    .unwrap()
                    .to_owned();
                let w = v.id().get::<i64>().unwrap().to_owned();
                (val, w)
            })
            .collect::<HashMap<String, Word>>();

        ans
    }

    fn get_pair(&self, key: GID) -> NewPair {
        let e = self.g.e(key);
        let v1 = e.clone().out_v().next().unwrap().unwrap();
        let v2 = e.clone().in_v().next().unwrap().unwrap();
        let p = e
            .properties("hash")
            .next()
            .unwrap()
            .unwrap()
            .get::<i64>()
            .unwrap()
            .to_owned();

        NewPair {
            first_word: v1.id().get::<i64>().unwrap().to_owned(),
            second_word: v2.id().get::<i64>().unwrap().to_owned(),
            pair_hash: p,
        }
    }

    fn get_phrase(&self, key: i64) -> Vec<Word> {
        self.phrases_by_hash[&key].to_owned()
    }

    fn get_orthotope(&self, key: i64) -> Ortho {
        self.orthos_by_hash[&key].to_owned()
    }

    fn insert_sentences(&mut self, sentences: &[models::NewSentence]) -> Vec<GID> {
        let existing = self
            .g
            .v(())
            .has_label("sentence")
            .has((
                "hash",
                P::within(sentences.iter().map(|x| x.sentence_hash).collect_vec()),
            ))
            .values("hash")
            .to_list()
            .unwrap()
            .iter()
            .map(|v| v.get::<i64>().unwrap().to_owned())
            .collect::<HashSet<i64>>();

        let new_ids = sentences
            .iter()
            .filter(|s| !existing.contains(&s.sentence_hash))
            .map(|sentence| {
                self.g
                    .add_v("sentence")
                    .property("hash", sentence.sentence_hash)
                    .property("sentence", &sentence.sentence)
                    .next()
                    .unwrap()
                    .unwrap()
                    .id()
                    .clone()
            })
            .collect_vec();

        new_ids
    }

    pub fn insert_todos(&mut self, domain: &str, hashes: Vec<i64>) {
        hashes.into_iter().for_each(|other| {
            self.todos.push(NewTodo {
                domain: domain.to_owned(),
                other,
                gid: GID::Int32(5),
            })
        })
    }

    pub fn insert_todos_with_gid(&mut self, domain: &str, hashes: Vec<GID>) {
        hashes.into_iter().for_each(|other| {
            self.todos.push(NewTodo {
                domain: domain.to_owned(),
                gid: other,
                other: 5,
            })
        })
    }

    fn get_sentence(&self, pk: GID) -> String {
        self.g
            .v(pk.clone())
            .clone()
            .values("sentence")
            .next()
            .unwrap()
            .unwrap()
            .get::<String>()
            .unwrap()
            .to_string()
    }

    fn insert_pairs(&mut self, to_insert: Vec<models::NewPair>) -> Vec<GID> {
        let existing = self
            .g
            .e(())
            .has_label("pair")
            .has((
                "hash",
                P::within(to_insert.iter().map(|x| x.pair_hash).collect_vec()),
            ))
            .values("hash")
            .to_list()
            .unwrap()
            .iter()
            .map(|v| v.get::<i64>().unwrap().to_owned())
            .collect::<HashSet<i64>>();

        let new_ids = to_insert
            .iter()
            .filter(|p| !existing.contains(&p.pair_hash))
            .map(|pair| {
                let v1 = self.g.v(pair.first_word).next().unwrap().unwrap();
                let v2 = self.g.v(pair.second_word).next().unwrap().unwrap();
                self.g
                    .add_e("pair")
                    .from(&v1)
                    .to(&v2)
                    .property("hash", pair.pair_hash)
                    .next()
                    .unwrap()
                    .unwrap()
                    .id()
                    .clone()
            })
            .collect_vec();

        new_ids
    }

    fn insert_phrases(&mut self, to_insert: Vec<models::NewPhrase>) -> Vec<i64> {
        let mut res = vec![];
        to_insert.into_iter().for_each(|new_phrase| {
            let inserted = self
                .phrases_by_head
                .entry(new_phrase.phrase_head)
                .or_default()
                .insert(new_phrase.words_hash);
            if inserted {
                res.push(new_phrase.words_hash);
                self.phrases_by_tail
                    .entry(new_phrase.phrase_tail)
                    .or_default()
                    .insert(new_phrase.words_hash);
                self.phrases_by_hash
                    .insert(new_phrase.words_hash, new_phrase.words);
            }
        });

        res
    }

    fn get_orthos_with_origin(&self, origin: Word) -> Vec<Ortho> {
        self.orthos_by_origin
            .get(&origin)
            .unwrap_or(&HashSet::default())
            .into_iter()
            .cloned()
            .collect_vec()
    }

    fn get_base_orthos_with_origin(&self, origin: Word) -> Vec<Ortho> {
        self.orthos_by_origin
            .get(&origin)
            .unwrap_or(&HashSet::default())
            .into_iter()
            .filter(|o| o.is_base())
            .cloned()
            .collect_vec()
    }

    fn get_ortho_with_origin_in(&self, origins: HashSet<Word>) -> Vec<Ortho> {
        origins
            .iter()
            .flat_map(|o| {
                self.orthos_by_origin
                    .get(o)
                    .unwrap_or(&HashSet::default())
                    .iter()
                    .cloned()
                    .collect_vec()
            })
            .collect()
    }

    fn insert_orthos(&mut self, to_insert: HashSet<NewOrthotope>) -> Vec<i64> {
        let mut res = vec![];
        to_insert.into_iter().for_each(|new_ortho| {
            let inserted_anew = self
                .orthos_by_hash
                .insert(new_ortho.info_hash, new_ortho.information.clone());
            if inserted_anew.is_none() {
                res.push(new_ortho.info_hash);
                new_ortho.hop.iter().for_each(|h| {
                    self.orthos_by_hop
                        .entry(*h)
                        .or_default()
                        .insert(new_ortho.clone());
                });

                new_ortho.contents.iter().for_each(|h| {
                    self.orthos_by_contents
                        .entry(*h)
                        .or_default()
                        .insert(new_ortho.clone());
                });

                self.orthos_by_origin
                    .entry(new_ortho.origin)
                    .or_default()
                    .insert(new_ortho.information);
            }
        });

        res
    }

    pub fn insert_book(&mut self, title: String, body: String) -> Book {
        let res = self
            .g
            .add_v("book")
            .property("title", title.clone())
            .property("body", body.clone())
            .next()
            .unwrap()
            .unwrap();
        let b = Book {
            id: res.id().clone(),
            title,
            body,
        };
        b
    }

    pub fn get_next_todo(&mut self) -> Option<NewTodo> {
        self.todos.pop()
    }
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

pub fn ortho_to_orthotope(ortho: &Ortho) -> NewOrthotope {
    let information = bincode::serialize(&ortho).expect("serialization should work");
    let origin = ortho.get_origin().to_owned();
    let hop = Vec::from_iter(ortho.get_hop());
    let contents = Vec::from_iter(ortho.get_contents());
    let info_hash = pair_todo_handler::data_vec_to_signed_int(&information);
    let base = ortho.is_base();
    NewOrthotope {
        information: ortho.clone(),
        origin,
        hop,
        contents,
        info_hash,
        base,
    }
}
