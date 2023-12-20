pub mod models;

use itertools::Itertools;
use maplit::hashset;
use redb::{
    Database, MultimapTableDefinition, ReadableMultimapTable, ReadableTable, TableDefinition,
};

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

use models::{NewBook, NewPair, NewTodo};
use std::collections::{HashMap, HashSet};
use std::{
    collections::hash_map::DefaultHasher,
    hash::{Hash, Hasher},
};

type Word = i32;
const BOOKS: TableDefinition<i64, Vec<u8>> = TableDefinition::new("books");
const VOCABULARY: TableDefinition<&str, i32> = TableDefinition::new("vocabulary");
const SENTENCES: TableDefinition<i64, &str> = TableDefinition::new("sentences");
const TODOS: MultimapTableDefinition<&str, i64> = MultimapTableDefinition::new("todos");

impl From<Vec<u8>> for NewBook {
    fn from(value: Vec<u8>) -> Self {
        bincode::deserialize(&value).unwrap()
    }
}

impl From<NewBook> for Vec<u8> {
    fn from(value: NewBook) -> Self {
        bincode::serialize(&value).unwrap()
    }
}

#[derive(Debug, Default)]
pub struct Holder {
    pairs_by_first: HashMap<Word, HashSet<NewPair>>,
    pairs_by_second: HashMap<Word, HashSet<NewPair>>,
    pairs_by_hash: HashMap<i64, NewPair>,
    phrases_by_head: HashMap<i64, HashSet<i64>>,
    phrases_by_tail: HashMap<i64, HashSet<i64>>,
    phrases_by_hash: HashMap<i64, Vec<Word>>,
    orthos_by_hash: HashMap<i64, Ortho>,
    orthos_by_hop: HashMap<Word, HashSet<NewOrthotope>>,
    orthos_by_contents: HashMap<Word, HashSet<NewOrthotope>>,
    orthos_by_origin: HashMap<Word, HashSet<Ortho>>,
}

impl Holder {
    pub fn new() -> Self {
        Holder::default()
    }

    pub fn get_stats(&self) {
        let todo_length = Database::create("pvac.redb")
            .unwrap()
            .begin_read()
            .unwrap()
            .open_multimap_table(TODOS)
            .unwrap()
            .len();

        dbg!(todo_length.unwrap());
        dbg!(&self.orthos_by_hash.len());
    }

    fn get_hashes_of_pairs_with_first_word(&self, firsts: Vec<Word>) -> HashSet<i64> {
        firsts
            .iter()
            .flat_map(|f| {
                self.pairs_by_first
                    .get(f)
                    .unwrap_or(&HashSet::default())
                    .iter()
                    .map(|x| x.pair_hash)
                    .collect::<HashSet<_>>()
            })
            .collect()
    }

    fn get_vocabulary_slice_with_words(&self, desired: HashSet<Word>) -> HashMap<Word, String> {
        Database::create("pvac.redb")
            .unwrap()
            .begin_read()
            .unwrap()
            .open_table(VOCABULARY)
            .unwrap()
            .iter()
            .unwrap()
            .into_iter()
            .filter(|x| desired.contains(&x.as_ref().unwrap().1.value()))
            .map(|mut x| {
                (
                    x.as_mut().unwrap().1.value().clone(),
                    x.unwrap().0.value().to_owned().clone(),
                )
            })
            .collect()
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
        from.iter()
            .flat_map(|f| {
                self.pairs_by_second
                    .get(f)
                    .unwrap_or(&HashSet::default())
                    .iter()
                    .map(|p| (p.first_word, p.second_word))
                    .collect_vec()
            })
            .collect()
    }

    fn get_words_of_pairs_with_first_word_in(&self, from: HashSet<Word>) -> HashSet<(Word, Word)> {
        from.iter()
            .flat_map(|f| {
                self.pairs_by_first
                    .get(f)
                    .unwrap_or(&HashSet::default())
                    .iter()
                    .map(|p| (p.first_word, p.second_word))
                    .collect_vec()
            })
            .collect()
    }

    fn get_hashes_and_words_of_pairs_with_first_word(
        &self,
        from: HashSet<Word>,
    ) -> HashSet<(Word, Word, i64)> {
        from.iter()
            .flat_map(|f| {
                self.pairs_by_first
                    .get(f)
                    .unwrap_or(&HashSet::default())
                    .iter()
                    .map(|p| (p.first_word, p.second_word, p.pair_hash))
                    .collect_vec()
            })
            .collect()
    }

    fn get_hashes_and_words_of_pairs_with_second_word(
        &self,
        from: HashSet<Word>,
    ) -> HashSet<(Word, Word, i64)> {
        from.iter()
            .flat_map(|f| {
                self.pairs_by_second
                    .get(f)
                    .unwrap_or(&HashSet::default())
                    .iter()
                    .map(|p| (p.first_word, p.second_word, p.pair_hash))
                    .collect_vec()
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
        seconds
            .iter()
            .flat_map(|s| {
                self.pairs_by_second
                    .get(s)
                    .unwrap_or(&HashSet::default())
                    .iter()
                    .map(|p| p.pair_hash)
                    .collect_vec()
            })
            .collect()
    }

    fn get_second_words_of_pairs_with_first_word(&self, first: Word) -> HashSet<Word> {
        self.pairs_by_first
            .get(&first)
            .unwrap_or(&HashSet::default())
            .iter()
            .map(|p| p.second_word)
            .collect()
    }

    fn get_first_words_of_pairs_with_second_word(&self, second: Word) -> HashSet<Word> {
        self.pairs_by_second
            .get(&second)
            .unwrap_or(&HashSet::default())
            .iter()
            .map(|p| p.first_word)
            .collect()
    }

    fn get_book(&self, pk: i64) -> NewBook {
        Database::create("pvac.redb")
            .unwrap()
            .begin_read()
            .unwrap()
            .open_table(BOOKS)
            .unwrap()
            .get(&pk)
            .unwrap()
            .unwrap()
            .value()
            .into()
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
        // todo make sure indices are right. Back to back inserts should count on

        let binding = Database::create("pvac.redb").unwrap();
        let db = binding.begin_write().unwrap();
        {
            let mut table = db.open_table(VOCABULARY).unwrap();

            let current: HashSet<String> = table
                .iter()
                .unwrap()
                .map(|x| x.unwrap().0.value().to_string())
                .collect();

            let words_to_insert: HashSet<String> =
                to_insert.iter().map(|w| w.word.clone()).collect();
            let new = words_to_insert.difference(&current).collect_vec();

            let current_index: usize = table.len().unwrap().try_into().unwrap();
            let f: usize = new.len();
            let new_indices = current_index..(current_index + f);

            words_to_insert.iter().zip(new_indices).for_each(|(k, v)| {
                let v_32: i32 = v.try_into().unwrap();
                table.insert(k.as_str(), v_32).unwrap();
            });
        }

        db.commit().unwrap();
    }

    fn get_vocabulary(&self, words: HashSet<String>) -> HashMap<String, Word> {
        let binding = Database::create("pvac.redb").unwrap();
        let db = binding.begin_read().unwrap();

        let table = db.open_table(VOCABULARY).unwrap();

        let ans = table
            .iter()
            .unwrap()
            .map(|mut x| {
                (
                    x.as_mut().unwrap().1.value().clone(),
                    x.unwrap().0.value().to_owned().clone(),
                )
            })
            .filter(|(_k, v)| words.contains(v))
            .map(|(k, v)| (v, k))
            .collect();
        ans
    }

    fn get_pair(&self, key: i64) -> NewPair {
        self.pairs_by_hash[&key].clone()
    }

    fn get_phrase(&self, key: i64) -> Vec<Word> {
        self.phrases_by_hash[&key].to_owned()
    }

    fn get_orthotope(&self, key: i64) -> Ortho {
        self.orthos_by_hash[&key].to_owned()
    }

    fn insert_sentences(&mut self, sentences: &[models::NewSentence]) -> Vec<i64> {
        let db: Database = Database::create("pvac.redb").unwrap();
        let mut new_sentences = Vec::default();
        let write_txn = db.begin_write().unwrap();
        {
            let mut table = write_txn.open_table(SENTENCES).unwrap();
            sentences.iter().for_each(|x| {
                let inserted_anew = table
                    .insert(x.sentence_hash, x.sentence.as_str())
                    .unwrap()
                    .is_none();
                if inserted_anew {
                    new_sentences.push(x.sentence_hash);
                }
            });
        }

        write_txn.commit().unwrap();
        new_sentences
    }

    pub fn insert_todos(&mut self, domain: &str, hashes: Vec<i64>) {
        // dbg!(domain, hashes.len());
        let db: Database = Database::create("pvac.redb").unwrap();
        let write_txn = db.begin_write().unwrap();
        {
            let mut table = write_txn.open_multimap_table(TODOS).unwrap();
            hashes.iter().for_each(|h| {
                table.insert(domain, h).unwrap();
            });
        }

        write_txn.commit().unwrap();
    }

    fn get_sentence(&self, pk: i64) -> String {
        Database::create("pvac.redb")
            .unwrap()
            .begin_read()
            .unwrap()
            .open_table(SENTENCES)
            .unwrap()
            .get(&pk)
            .unwrap()
            .unwrap()
            .value()
            .into()
    }

    fn insert_pairs(&mut self, to_insert: Vec<models::NewPair>) -> Vec<i64> {
        let mut res = vec![];
        to_insert.iter().for_each(|new_pair| {
            let inserted = self
                .pairs_by_first
                .entry(new_pair.first_word)
                .or_default()
                .insert(new_pair.clone());
            if inserted {
                res.push(new_pair.pair_hash);
                self.pairs_by_hash
                    .insert(new_pair.pair_hash, new_pair.clone());
                self.pairs_by_second
                    .entry(new_pair.second_word)
                    .or_default()
                    .insert(new_pair.clone());
            }
        });

        res
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

    pub fn insert_book(&mut self, title: String, body: String) -> i64 {
        let b = NewBook {
            title: title.clone(),
            body,
        };
        let b_bytes: Vec<u8> = b.clone().into();
        let id: i64 = string_to_signed_int(&title).try_into().unwrap();
        let db: Database = Database::create("pvac.redb").unwrap();
        let write_txn = db.begin_write().unwrap();
        {
            let mut table = write_txn.open_table(BOOKS).unwrap();
            table.insert(id, b_bytes).unwrap();
        }

        write_txn.commit().unwrap();
        id
    }

    fn find_next_todo(&mut self) -> Option<NewTodo> {
        let dfs = vec![
            "books",
            "sentences",
            "pairs",
            "pair_up",
            "ex_nihilo_ffbb",
            "ex_nihilo_fbbf",
            "up_by_origin",
            "up_by_hop",
            "up_by_contents",
            "phrases",
            "phrase_by_origin",
            "phrase_by_hop",
            "phrase_by_contents",
            "orthotopes",
            "ortho_up",
            "ortho_up_forward",
            "ortho_up_back",
            "ortho_over",
            "ortho_over_forward",
            "ortho_over_back",
        ];

        let db: Database = Database::create("pvac.redb").unwrap();
        let read_txn = db.begin_read().unwrap();
        {
            let table = read_txn.open_multimap_table(TODOS).unwrap();

            for domain in dfs {
                let mut res_set = table.get(domain).unwrap();
                let val = res_set.next();
                if val.is_some() {
                    let idx = val.unwrap().unwrap().value();

                    return Some(NewTodo {
                        domain: domain.to_owned(),
                        other: idx,
                    });
                }
            }
        };
        None
    }

    pub fn get_next_todo(&mut self) -> Option<NewTodo> {
        let next_todo = self.find_next_todo();

        if next_todo.is_some() {
            let current_todo = next_todo.clone().unwrap();
            let db: Database = Database::create("pvac.redb").unwrap();
            let mut write_txn = db.begin_write().unwrap();
            {
                write_txn.set_durability(redb::Durability::None);
                let mut table = write_txn.open_multimap_table(TODOS).unwrap();
                table
                    .remove(current_todo.domain.as_str(), current_todo.other)
                    .unwrap();
            }
            write_txn.commit().unwrap();
            next_todo
        } else {
            None
        }
    }
}

pub fn get_hashes_of_pairs_with_words_in(
    holder: &mut Holder,
    first_words: HashSet<Word>,
    second_words: HashSet<Word>,
) -> HashSet<i64> {
    let firsts: HashSet<i64> =
        holder.get_hashes_of_pairs_with_first_word(Vec::from_iter(first_words));
    let seconds: HashSet<i64> =
        holder.get_hashes_of_pairs_with_second_word(Vec::from_iter(second_words));

    firsts.intersection(&seconds).cloned().collect()
}

pub fn get_hashes_and_words_of_pairs_with_words_in(
    holder: &Holder,
    first_words: HashSet<Word>,
    second_words: HashSet<Word>,
) -> (HashSet<Word>, HashSet<Word>, HashSet<i64>) {
    let firsts: HashSet<(Word, Word, i64)> =
        holder.get_hashes_and_words_of_pairs_with_first_word(first_words);
    let seconds: HashSet<(Word, Word, i64)> =
        holder.get_hashes_and_words_of_pairs_with_second_word(second_words);

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
    holder
        .get_phrases_matching(all_phrases)
        .iter()
        .cloned()
        .collect()
}

fn project_forward_batch(holder: &Holder, from: HashSet<Word>) -> HashSet<(Word, Word)> {
    holder.get_words_of_pairs_with_first_word_in(from)
}

fn project_backward_batch(holder: &Holder, from: HashSet<Word>) -> HashSet<(Word, Word)> {
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

fn project_forward(holder: &Holder, from: Word) -> HashSet<Word> {
    holder.get_second_words_of_pairs_with_first_word(from)
}

fn project_backward(holder: &Holder, from: Word) -> HashSet<Word> {
    holder.get_first_words_of_pairs_with_second_word(from)
}

pub fn insert_orthotopes(holder: &mut Holder, new_orthos: HashSet<NewOrthotope>) -> Vec<i64> {
    holder.insert_orthos(new_orthos)
}

pub fn get_ortho_by_origin(holder: &mut Holder, o: Word) -> Vec<Ortho> {
    holder.get_orthos_with_origin(o)
}

pub fn get_base_ortho_by_origin(holder: &mut Holder, o: Word) -> Vec<Ortho> {
    holder.get_base_orthos_with_origin(o)
}

pub fn get_ortho_by_origin_batch(holder: &mut Holder, o: HashSet<Word>) -> Vec<Ortho> {
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
        information: ortho.clone(),
        origin,
        hop,
        contents,
        info_hash,
        base,
    }
}

fn get_ortho_by_hop(holder: &Holder, other_hop: Vec<Word>) -> Vec<Ortho> {
    holder.get_orthos_with_hops_overlapping(other_hop)
}

fn get_base_ortho_by_hop(holder: &Holder, other_hop: Vec<Word>) -> Vec<Ortho> {
    holder.get_base_orthos_with_hops_overlapping(other_hop)
}

fn get_ortho_by_contents(holder: &Holder, other_contents: Vec<Word>) -> Vec<Ortho> {
    holder.get_orthos_with_contents_overlapping(other_contents)
}

fn get_base_ortho_by_contents(holder: &Holder, other_contents: Vec<Word>) -> Vec<Ortho> {
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

pub(crate) fn phrase_exists_db_filter_head(holder: &Holder, left: HashSet<i64>) -> HashSet<i64> {
    holder.get_phrase_hash_with_phrase_head_matching(left)
}

pub(crate) fn phrase_exists_db_filter_tail(holder: &Holder, right: HashSet<i64>) -> HashSet<i64> {
    holder.get_phrase_hash_with_phrase_tail_matching(right)
}

fn get_relevant_vocabulary(holder: &Holder, words: HashSet<String>) -> HashMap<String, Word> {
    holder.get_vocabulary(words)
}

pub fn get_relevant_vocabulary_reverse(
    holder: &Holder,
    words: HashSet<Word>,
) -> HashMap<Word, String> {
    holder.get_vocabulary_slice_with_words(words)
}
