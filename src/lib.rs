pub mod models;

use itertools::Itertools;
use maplit::hashset;
use redb::{
    Database, MultimapTableDefinition, ReadableMultimapTable, ReadableTable, TableDefinition,
};
use up_handler::total_vocabulary;

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

use models::{DBNewTodo, NewBook, NewPair, NewTodo};
use std::collections::{HashMap, HashSet};
use std::{
    collections::hash_map::DefaultHasher,
    hash::{Hash, Hasher},
};

type Word = i64;
const BOOKS: TableDefinition<i64, Vec<u8>> = TableDefinition::new("books");
const VOCABULARY: TableDefinition<&str, Word> = TableDefinition::new("vocabulary");
const SENTENCES: TableDefinition<i64, &str> = TableDefinition::new("sentences");
const TODOS: MultimapTableDefinition<&str, &[u8]> = MultimapTableDefinition::new("todos");
const PAIRS_BY_FIRST: MultimapTableDefinition<Word, &[u8]> =
    MultimapTableDefinition::new("pairs_by_first");
const PAIRS_BY_SECOND: MultimapTableDefinition<Word, &[u8]> =
    MultimapTableDefinition::new("pairs_by_second");
const PAIRS_BY_HASH: TableDefinition<i64, Vec<u8>> = TableDefinition::new("pairs_by_hash");
const PHRASES_BY_HEAD: MultimapTableDefinition<i64, i64> =
    MultimapTableDefinition::new("phrases_by_head");
const PHRASES_BY_TAIL: MultimapTableDefinition<i64, i64> =
    MultimapTableDefinition::new("phrases_by_tail");
const PHRASES_BY_HASH: TableDefinition<i64, Vec<Word>> = TableDefinition::new("phrases_by_hash");
const ORTHOS_BY_HASH: TableDefinition<i64, Vec<u8>> = TableDefinition::new("orthos_by_hash");
const ORTHOS_BY_HOP: MultimapTableDefinition<Word, &[u8]> =
    MultimapTableDefinition::new("orthos_by_hop");
const ORTHOS_BY_CONTENTS: MultimapTableDefinition<Word, &[u8]> =
    MultimapTableDefinition::new("orthos_by_contents");
const ORTHOS_BY_ORIGIN: MultimapTableDefinition<Word, &[u8]> =
    MultimapTableDefinition::new("orthos_by_origin");

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

impl From<Vec<u8>> for DBNewTodo {
    fn from(value: Vec<u8>) -> Self {
        bincode::deserialize(&value).unwrap()
    }
}

impl From<DBNewTodo> for Vec<u8> {
    fn from(value: DBNewTodo) -> Self {
        bincode::serialize(&value).unwrap()
    }
}

impl From<Vec<u8>> for NewPair {
    fn from(value: Vec<u8>) -> Self {
        bincode::deserialize(&value).unwrap()
    }
}

impl From<NewPair> for Vec<u8> {
    fn from(value: NewPair) -> Self {
        bincode::serialize(&value).unwrap()
    }
}

impl From<Vec<u8>> for Ortho {
    fn from(value: Vec<u8>) -> Self {
        bincode::deserialize(&value).unwrap()
    }
}

impl From<Ortho> for Vec<u8> {
    fn from(value: Ortho) -> Self {
        bincode::serialize(&value).unwrap()
    }
}

impl From<Vec<u8>> for NewOrthotope {
    fn from(value: Vec<u8>) -> Self {
        bincode::deserialize(&value).unwrap()
    }
}

impl From<NewOrthotope> for Vec<u8> {
    fn from(value: NewOrthotope) -> Self {
        bincode::serialize(&value).unwrap()
    }
}

#[derive(Debug)]
pub struct Holder {
    undone_todos: Vec<NewTodo>,
    done_todos: Vec<NewTodo>,
    db_name: String,
}

impl Holder {
    pub fn new(db_name: String) -> Self {
        Holder{ undone_todos: vec![], done_todos: vec![], db_name }
    }

    pub fn get_stats(&self) {
        let todo_length = self.undone_todos.len() + self.done_todos.len();

        dbg!(todo_length);
        dbg!(self.get_db()
            .begin_read()
            .unwrap()
            .open_table(ORTHOS_BY_HASH)
            .unwrap()
            .len())
        .unwrap();
    }

    fn get_hashes_of_pairs_with_first_word(&self, firsts: Vec<Word>) -> HashSet<i64> {
        let binding = self.get_db();
        let binding = binding
        .begin_read()
        .unwrap();
        let read_only_multimap_table = &binding
        .open_multimap_table(PAIRS_BY_FIRST)
        .unwrap();


        firsts
            .iter()
            .flat_map(|f| {
                read_only_multimap_table
                    .get(f)
                    .unwrap()
                    .map(|x| Into::<NewPair>::into(x.unwrap().value().to_vec()).pair_hash)
                    .collect::<Vec<_>>()
            })
            .collect()
    }

    fn get_vocabulary_slice_with_words(&self, desired: HashSet<Word>) -> HashMap<Word, String> {
        self.get_db()
            .begin_write()
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
        let binding = self.get_db();
        let binding = binding
                    .begin_read()
                    .unwrap();
        let read_only_multimap_table = &binding
                    .open_multimap_table(ORTHOS_BY_HOP)
                    .unwrap();
        hop.iter()
            .flat_map(|f| {
                read_only_multimap_table
                    .get(f)
                    .unwrap()
                    .map(|x| Into::<NewOrthotope>::into(x.unwrap().value().to_vec()).information)
                    .collect::<Vec<_>>()
            })
            .collect()
    }

    fn get_base_orthos_with_hops_overlapping(&self, hop: Vec<Word>) -> Vec<Ortho> {
        let binding = self.get_db();
        let binding = binding
                    .begin_read()
                    .unwrap();
        let read_only_multimap_table = &binding
                    .open_multimap_table(ORTHOS_BY_HOP)
                    .unwrap();
        hop.iter()
            .flat_map(|f| {
                read_only_multimap_table
                    .get(f)
                    .unwrap()
                    .map(|x| Into::<NewOrthotope>::into(x.unwrap().value().to_vec()))
                    .filter(|o| o.base)
                    .map(|o| o.information)
                    .collect::<Vec<_>>()
            })
            .collect()
    }

    fn get_orthos_with_contents_overlapping(&self, other_contents: Vec<Word>) -> Vec<Ortho> {
        let binding = self.get_db();
        let binding = binding
                    .begin_write()
                    .unwrap();
        let multimap_table = &binding
                    .open_multimap_table(ORTHOS_BY_CONTENTS)
                    .unwrap();
        other_contents
            .iter()
            .flat_map(|f| {
                multimap_table
                    .get(f)
                    .unwrap()
                    .map(|x| Into::<NewOrthotope>::into(x.unwrap().value().to_vec()).information)
                    .collect::<Vec<_>>()
            })
            .collect()
    }

    fn get_base_orthos_with_contents_overlapping(&self, other_contents: Vec<Word>) -> Vec<Ortho> {
        let binding = self.get_db();
        let binding = binding
                    .begin_read()
                    .unwrap();
        let read_only_multimap_table = &binding
                    .open_multimap_table(ORTHOS_BY_CONTENTS)
                    .unwrap();
        other_contents
            .iter()
            .flat_map(|f| {
                read_only_multimap_table
                    .get(f)
                    .unwrap()
                    .map(|x| Into::<NewOrthotope>::into(x.unwrap().value().to_vec()))
                    .filter(|o| o.base)
                    .map(|o| o.information)
                    .collect::<Vec<_>>()
            })
            .collect()
    }

    fn get_words_of_pairs_with_second_word_in(&self, from: HashSet<Word>) -> HashSet<(Word, Word)> {
        let binding = self.get_db();
        let binding = binding
                    .begin_read()
                    .unwrap();
        let read_only_multimap_table = &binding
                    .open_multimap_table(PAIRS_BY_SECOND)
                    .unwrap();
        from.iter()
            .flat_map(|f| {
                
                read_only_multimap_table
                    .get(f)
                    .unwrap()
                    .map(|x| {
                        let p = Into::<NewPair>::into(x.unwrap().value().to_vec());
                        (p.first_word, p.second_word)
                    })
                    .collect::<Vec<_>>()
            })
            .collect()
    }

    fn get_words_of_pairs_with_first_word_in(&self, from: HashSet<Word>) -> HashSet<(Word, Word)> {
        let binding = self.get_db();
        let binding = binding
                    .begin_read()
                    .unwrap();
        let read_only_multimap_table = &binding
                    .open_multimap_table(PAIRS_BY_FIRST)
                    .unwrap();
        from.iter()
            .flat_map(|f| {
                read_only_multimap_table
                    .get(f)
                    .unwrap()
                    .map(|x| {
                        let p = Into::<NewPair>::into(x.unwrap().value().to_vec());
                        (p.first_word, p.second_word)
                    })
                    .collect::<Vec<_>>()
            })
            .collect()
    }

    fn get_hashes_and_words_of_pairs_with_first_word(
        &self,
        from: HashSet<Word>,
    ) -> HashSet<(Word, Word, i64)> {
        let binding = self.get_db();
        let binding = binding
                    .begin_read()
                    .unwrap();
        let read_only_multimap_table = &binding
                    .open_multimap_table(PAIRS_BY_FIRST)
                    .unwrap();
        from.iter()
            .flat_map(|f| {
                read_only_multimap_table
                    .get(f)
                    .unwrap()
                    .map(|x| {
                        let p = Into::<NewPair>::into(x.unwrap().value().to_vec());
                        (p.first_word, p.second_word, p.pair_hash)
                    })
                    .collect::<Vec<_>>()
            })
            .collect()
    }

    fn get_hashes_and_words_of_pairs_with_second_word(
        &self,
        from: HashSet<Word>,
    ) -> HashSet<(Word, Word, i64)> {
        let binding = self.get_db();
        let binding = binding
                    .begin_read()
                    .unwrap();
        let read_only_multimap_table = &binding
                    .open_multimap_table(PAIRS_BY_SECOND)
                    .unwrap();
        from.iter()
            .flat_map(|f| {
                
                read_only_multimap_table
                    .get(f)
                    .unwrap()
                    .map(|x| {
                        let p = Into::<NewPair>::into(x.unwrap().value().to_vec());
                        (p.first_word, p.second_word, p.pair_hash)
                    })
                    .collect::<Vec<_>>()
            })
            .collect()
    }

    fn get_phrase_hash_with_phrase_head_matching(&self, left: HashSet<i64>) -> HashSet<i64> {
        let binding = self.get_db();
        let binding = binding
                    .begin_read()
                    .unwrap();
        let read_only_multimap_table = &binding
                    .open_multimap_table(PHRASES_BY_HEAD)
                    .unwrap();
        left.iter()
            .flat_map(|f| {
                
                read_only_multimap_table
                    .get(f)
                    .unwrap()
                    .map(|x| x.unwrap().value())
                    .collect::<Vec<_>>()
            })
            .collect()
    }

    fn get_phrase_hash_with_phrase_tail_matching(&self, left: HashSet<i64>) -> HashSet<i64> {
        let binding = self.get_db();
        let binding = binding
                    .begin_read()
                    .unwrap();
        let read_only_multimap_table = &binding
                    .open_multimap_table(PHRASES_BY_TAIL)
                    .unwrap();
        left.iter()
            .flat_map(|f| {
                
                read_only_multimap_table
                    .get(f)
                    .unwrap()
                    .map(|x| x.unwrap().value())
                    .collect::<Vec<_>>()
            })
            .collect()
    }

    fn get_phrases_matching(&self, phrases: HashSet<i64>) -> HashSet<i64> {
        let binding = self.get_db();
        let binding = binding.begin_read().unwrap();
        let read_only_table = binding.open_table(PHRASES_BY_HASH).unwrap();
        phrases
            .iter()
            .flat_map(|f| {
                read_only_table
                    .get(f)
                    .unwrap()
                    .iter()
                    .map(|x| vec_of_words_to_big_int(x.value()))
                    .collect_vec()
            })
            .collect()
    }

    fn get_hashes_of_pairs_with_second_word(&self, seconds: Vec<Word>) -> HashSet<i64> {
        let binding = self.get_db();
        let binding = binding
                    .begin_read()
                    .unwrap();
        let read_only_multimap_table = &binding
                    .open_multimap_table(PAIRS_BY_SECOND)
                    .unwrap();
        seconds
            .iter()
            .flat_map(|f| {
                
                read_only_multimap_table
                    .get(f)
                    .unwrap()
                    .map(|x| {
                        let p = Into::<NewPair>::into(x.unwrap().value().to_vec());
                        p.pair_hash
                    })
                    .collect::<Vec<_>>()
            })
            .collect()
    }

    fn get_second_words_of_pairs_with_first_word(&self, first: Word) -> HashSet<Word> {
        self.get_db()
            .begin_read()
            .unwrap()
            .open_multimap_table(PAIRS_BY_FIRST)
            .unwrap()
            .get(first)
            .unwrap()
            .map(|x| {
                let p = Into::<NewPair>::into(x.unwrap().value().to_vec());
                p.second_word
            })
            .collect()
    }

    fn get_first_words_of_pairs_with_second_word(&self, second: Word) -> HashSet<Word> {
        self.get_db()
            .begin_read()
            .unwrap()
            .open_multimap_table(PAIRS_BY_SECOND)
            .unwrap()
            .get(second)
            .unwrap()
            .map(|x| {
                let p = Into::<NewPair>::into(x.unwrap().value().to_vec());
                p.first_word
            })
            .collect()
    }

    fn get_book(&self, pk: i64) -> NewBook {
        self.get_db()
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

        let binding = self.get_db();
        let db = binding.begin_write().unwrap();
        {
            let mut table = db.open_table(VOCABULARY).unwrap();

            to_insert.iter().for_each(|x| {
                table.insert(x.word.as_str(), x.word_hash).unwrap();
            });
        }

        db.commit().unwrap();
    }

    fn get_vocabulary(&self, words: HashSet<String>) -> HashMap<String, Word> {
        let binding = self.get_db();
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
        self.get_db()
            .begin_read()
            .unwrap()
            .open_table(PAIRS_BY_HASH)
            .unwrap()
            .get(key)
            .unwrap()
            .unwrap()
            .value()
            .into()
    }

    fn get_phrase(&self, key: i64) -> Vec<Word> {
        self.get_db()
            .begin_read()
            .unwrap()
            .open_table(PHRASES_BY_HASH)
            .unwrap()
            .get(key)
            .unwrap()
            .unwrap()
            .value()
    }

    fn get_orthotope(&self, key: i64) -> Ortho {
        self.get_db()
            .begin_read()
            .unwrap()
            .open_table(ORTHOS_BY_HASH)
            .unwrap()
            .get(key)
            .unwrap()
            .unwrap()
            .value()
            .into()
    }

    fn insert_sentences(&mut self, sentences: &[models::NewSentence]) -> Vec<i64> {
        let db: Database = self.get_db();
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
        hashes.iter().for_each(|h| {
            let new_todo = NewTodo {
                domain: domain.to_string(),
                other: *h,
            };

            self.undone_todos.push(new_todo);
        });
    }

    fn get_sentence(&self, pk: i64) -> String {
        self.get_db()
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
        let db: Database = self.get_db();
        let write_txn = db.begin_write().unwrap();
        let mut res = vec![];
        {
            let mut first_table = write_txn.open_multimap_table(PAIRS_BY_FIRST).unwrap();
            let mut second_table = write_txn.open_multimap_table(PAIRS_BY_SECOND).unwrap();
            let mut hash_table = write_txn.open_table(PAIRS_BY_HASH).unwrap();

            to_insert.iter().for_each(|new_pair| {
                let rhs: &[u8] = &Into::<Vec<u8>>::into(new_pair.clone());
                let inserted = !first_table.insert(new_pair.first_word, rhs).unwrap();
                if inserted {
                    res.push(new_pair.pair_hash);
                    hash_table
                        .insert(new_pair.pair_hash, Into::<Vec<u8>>::into(new_pair.clone()))
                        .unwrap();
                    second_table.insert(new_pair.second_word, rhs).unwrap();
                }
            });
        }

        write_txn.commit().unwrap();
        res
    }

    // todo implement resume logic in the event of crash. Currently duplicate books will be ignored

    fn insert_phrases(&mut self, to_insert: Vec<models::NewPhrase>) -> Vec<i64> {
        let db: Database = self.get_db();
        let write_txn = db.begin_write().unwrap();
        let mut res = vec![];

        {
            let mut first_table = write_txn.open_multimap_table(PHRASES_BY_HEAD).unwrap();
            let mut second_table = write_txn.open_multimap_table(PHRASES_BY_TAIL).unwrap();
            let mut hash_table = write_txn.open_table(PHRASES_BY_HASH).unwrap();

            to_insert.iter().for_each(|new_phrase| {
                let inserted = !first_table
                    .insert(new_phrase.phrase_head, new_phrase.words_hash)
                    .unwrap();
                if inserted {
                    res.push(new_phrase.words_hash);
                    second_table
                        .insert(new_phrase.phrase_tail, new_phrase.words_hash)
                        .unwrap();
                    hash_table
                        .insert(new_phrase.words_hash, new_phrase.words.clone())
                        .unwrap();
                }
            });
        }

        write_txn.commit().unwrap();

        res
    }

    fn get_orthos_with_origin(&self, origin: Word) -> Vec<Ortho> {
        self.get_db()
            .begin_read()
            .unwrap()
            .open_multimap_table(ORTHOS_BY_ORIGIN)
            .unwrap()
            .get(origin)
            .unwrap()
            .map(|x| Into::<NewOrthotope>::into(x.unwrap().value().to_vec()))
            .map(|o| o.information)
            .collect::<Vec<_>>()
    }

    fn get_base_orthos_with_origin(&self, origin: Word) -> Vec<Ortho> {
        self.get_db()
            .begin_read()
            .unwrap()
            .open_multimap_table(ORTHOS_BY_ORIGIN)
            .unwrap()
            .get(origin)
            .unwrap()
            .map(|x| Into::<NewOrthotope>::into(x.unwrap().value().to_vec()))
            .filter(|o| o.base)
            .map(|o| o.information)
            .collect::<Vec<_>>()
    }

    fn get_ortho_with_origin_in(&self, origins: HashSet<Word>) -> Vec<Ortho> {
        let binding = self.get_db();
        let binding = binding
                    .begin_read()
                    .unwrap();
        let read_only_multimap_table = &binding
                    .open_multimap_table(ORTHOS_BY_ORIGIN)
                    .unwrap();
        origins
            .iter()
            .flat_map(|f| {
                
                read_only_multimap_table
                    .get(f)
                    .unwrap()
                    .map(|x| Into::<NewOrthotope>::into(x.unwrap().value().to_vec()))
                    .filter(|o| o.base)
                    .map(|o| o.information)
                    .collect::<Vec<_>>()
            })
            .collect()
    }

    fn insert_orthos(&mut self, to_insert: HashSet<NewOrthotope>) -> Vec<i64> {
        let db: Database = self.get_db();
        let write_txn = db.begin_write().unwrap();
        let mut res = vec![];

        {
            let mut hash_table = write_txn.open_table(ORTHOS_BY_HASH).unwrap();
            let mut hop_table = write_txn.open_multimap_table(ORTHOS_BY_HOP).unwrap();
            let mut contents_table = write_txn.open_multimap_table(ORTHOS_BY_CONTENTS).unwrap();
            let mut origin_table = write_txn.open_multimap_table(ORTHOS_BY_ORIGIN).unwrap();

            to_insert.iter().for_each(|new_ortho| {
                let inserted = hash_table
                    .insert(
                        new_ortho.info_hash,
                        Into::<Vec<u8>>::into(new_ortho.information.clone()),
                    )
                    .unwrap()
                    .is_none();
                if inserted {
                    res.push(new_ortho.info_hash);
                    new_ortho.hop.iter().for_each(|h| {
                        let rhs: &[u8] = &Into::<Vec<u8>>::into(new_ortho.clone());
                        hop_table.insert(h, rhs).unwrap();
                    });

                    new_ortho.contents.iter().for_each(|h| {
                        let rhs: &[u8] = &Into::<Vec<u8>>::into(new_ortho.clone());
                        contents_table.insert(h, rhs).unwrap();
                    });

                    let rhs: &[u8] = &Into::<Vec<u8>>::into(new_ortho.clone());
                    origin_table.insert(new_ortho.origin, rhs).unwrap();
                }
            });
        }

        write_txn.commit().unwrap();

        res
    }

    pub fn insert_book(&mut self, title: String, body: String) -> i64 {
        let b = NewBook {
            title: title.clone(),
            body,
        };
        let b_bytes: Vec<u8> = b.clone().into();
        let id: i64 = string_to_signed_int(&title).try_into().unwrap();
        let db: Database = self.get_db();
        let write_txn = db.begin_write().unwrap();
        {
            let mut table = write_txn.open_table(BOOKS).unwrap();
            table.insert(id, b_bytes).unwrap();
        }

        write_txn.commit().unwrap();
        id
    }

    pub fn get_next_todo(&mut self) -> Option<NewTodo> {
        self.undone_todos.pop()
    }

    pub fn save_todos(&mut self) {
        let db: Database = self.get_db();
        let write_txn = db.begin_write().unwrap();
        {
            let mut table = write_txn.open_multimap_table(TODOS).unwrap();

            self.undone_todos.iter().for_each(|ct| {
                let to_insert: Vec<u8> = DBNewTodo {
                    domain: ct.domain.clone(),
                    other: ct.other,
                    done: false,
                }
                .into();
                let to_insert_ref: &[u8] = &to_insert;
                table.insert("todos", to_insert_ref).unwrap();
            });

            self.done_todos.drain(..).for_each(|ct| {
                let to_insert: Vec<u8> = DBNewTodo {
                    domain: ct.domain.clone(),
                    other: ct.other,
                    done: true,
                }
                .into();
                let to_insert_ref: &[u8] = &to_insert;
                table.insert("todos", to_insert_ref).unwrap();
            });
        }

        write_txn.commit().unwrap();
    }

    pub fn unprocessed_todos_exist(&mut self) -> bool {
        self.rehydrate_todos();

        !self.undone_todos.is_empty()
    }

    pub fn rehydrate_todos(&mut self) {
        let db: Database = self.get_db();
        let read_txn = db.begin_write().unwrap();
        let table = read_txn.open_multimap_table(TODOS).unwrap();
        table.iter().unwrap().for_each(|td| {
            td.unwrap().1.for_each(|t| {
                let todo: DBNewTodo = t.unwrap().value().to_vec().into();
                if !todo.done {
                    self.undone_todos.push(NewTodo {
                        domain: todo.domain,
                        other: todo.other,
                    })
                }
            });
        });
    }

    pub fn complete_todo(&mut self, current: NewTodo) {
            self.done_todos.push(current);
    }

    fn get_db(&self) -> Database {
        Database::create(&self.db_name)
            .unwrap()
    }

    pub fn get_merged_todos(&self, lhs: String, rhs: String) -> Vec<NewTodo> {
        let lhs = Holder::new(lhs);
        let rhs = Holder::new(rhs);

        let mut replay_todos: Vec<NewTodo> = vec![];
        let all_lhs_todos = lhs.get_all_todos();
        let all_rhs_todos = rhs.get_all_todos();

        for todo in all_lhs_todos.clone() {
            if rhs.has_todo(&todo, &all_rhs_todos) {
                continue;
            }

            let lhs_friends = lhs.get_friends(&todo);
            let rhs_friends = rhs.get_friends(&todo);

            if self.friends_in_both(&lhs_friends, &rhs_friends) {
                continue;
            }

            if self.lhs_friends_share_provenance(&lhs_friends, &rhs_friends) {
                continue;
            }

            replay_todos.push(todo);
        }

        // todo reuse functionality for finding friends in actual search
        
        for todo in all_rhs_todos.clone() {
            if lhs.has_todo(&todo, &all_lhs_todos) {
                continue;
            }

            let lhs_friends = lhs.get_friends(&todo);
            let rhs_friends = rhs.get_friends(&todo);

            if self.friends_in_both(&lhs_friends, &rhs_friends) {
                continue;
            }

            if self.rhs_friends_share_provenance(&lhs_friends, &rhs_friends) {
                continue;
            }

            replay_todos.push(todo);
        }
        replay_todos
    }

    // todo stop passing function pointers

    fn get_all_todos(&self) -> Vec<NewTodo> {
        let mut res = vec![];
        let db: Database = self.get_db();
        let read_txn = db.begin_write().unwrap();
        let table = read_txn.open_multimap_table(TODOS).unwrap();
        table.iter().unwrap().for_each(|td| {
            td.unwrap().1.for_each(|t| {
                let todo: DBNewTodo = t.unwrap().value().to_vec().into();
                if !todo.done {
                    res.push(NewTodo {
                        domain: todo.domain,
                        other: todo.other,
                    })
                }
            });
        });
        res
    }

    fn has_todo(&self, todo: &NewTodo, others: &Vec<NewTodo>) -> bool {
        others.contains(todo)
    }

    fn get_friends(&self, todo: &NewTodo) -> Vec<NewTodo> {
        match todo.domain.as_str() {
            "books" => vec![],
            "sentences" => vec![],
            "pairs" => vec![],
            "ex_nihilo_ffbb" => self.get_friends_ffbb(todo.other.clone()),
            "ex_nihilo_fbbf" => self.get_friends_fbbf(todo.other.clone()),
            "pair_up" => vec![],
            "up_by_origin" => self.get_friends_up_by_origin(todo.other.clone()),
            "up_by_hop" => self.get_friends_up_by_hop(todo.other.clone()),
            "up_by_contents" => self.get_friends_up_by_contents(todo.other.clone()),
            "orthotopes" => vec![],
            "ortho_up" => vec![],
            "ortho_up_forward" => self.get_friends_up_forward(todo.other.clone()),
            "ortho_up_back" => self.get_friends_up_back(todo.other.clone()),
            "ortho_over" => vec![],
            "ortho_over_forward" => self.get_friends_over_forward(todo.other.clone()),
            "ortho_over_back" =>  self.get_friends_over_backward(todo.other.clone()),
            "phrases" => vec![],
            "phrase_by_origin" => self.get_friends_over_by_origin(todo.other.clone()),
            "phrase_by_hop" => self.get_friends_over_by_hop(todo.other.clone()),
            "phrase_by_contents" => self.get_friends_over_by_contents(todo.other.clone()),
            _ => panic!(),
        }
    }

    fn friends_in_both(&self, lhs_friends: &Vec<NewTodo>, rhs_friends: &Vec<NewTodo>) -> bool {
        let lhs: HashSet<&NewTodo> = HashSet::from_iter(lhs_friends);
        lhs.symmetric_difference(&HashSet::from_iter(rhs_friends)).next().is_none()
    }

    pub fn write_todos_back(&mut self, todos: Vec<NewTodo>) {
        self.undone_todos = todos;
    }

    pub fn merge_dbs(&self, lhs: String, rhs: String) {
        let db: Database = self.get_db();
        let read_txn = db.begin_write().unwrap();
        let table = read_txn.open_table(BOOKS).unwrap();
        table.iter().unwrap().for_each(|x| {
            let k = x.unwrap().0.value();
            let v = x.unwrap().1.value();

            
        })

//         BOOKS
// VOCABULARY
// SENTENCES
// TODOS
// PAIRS_BY_FIRST
// PAIRS_BY_SECOND
// PAIRS_BY_HASH
// PHRASES_BY_HEAD
// PHRASES_BY_TAIL
// PHRASES_BY_HASH
// ORTHOS_BY_HASH
// ORTHOS_BY_HOP
// ORTHOS_BY_CONTENTS
// ORTHOS_BY_ORIGIN
        
    }

    fn get_friends_ffbb(&self, other: i64) -> Vec<NewTodo> {
        let p = self.get_pair(other);
        self.ffbb(p.first_word, p.second_word).iter().map(|o| { 
            NewTodo {
                domain: "orthotopes".to_string(),
                other: ortho_to_orthotope(o).info_hash
            }
        }).collect_vec()
    }

    fn get_friends_fbbf(&self, other: i64) -> Vec<NewTodo> {
        let p = self.get_pair(other);
        self.fbbf(p.first_word, p.second_word).iter().map(|o| { 
            NewTodo {
                domain: "orthotopes".to_string(),
                other: ortho_to_orthotope(o).info_hash
            }
        }).collect_vec()
    }

    fn get_friends_up_by_origin(&self, other: i64) -> Vec<NewTodo> {
        let p = self.get_pair(other);
        let los = &self.get_base_orthos_with_origin(p.first_word);
        let ros = &self.get_base_orthos_with_origin(p.second_word);
        los.iter().map(|o| {
            NewTodo {
                domain: "orthotopes".to_string(),
                other: ortho_to_orthotope(o).info_hash
            }
        }).chain(ros.iter().map(|o|{
            NewTodo {
                domain: "orthotopes".to_string(),
                other: ortho_to_orthotope(o).info_hash
            }
        })).chain(get_hashes_of_pairs_with_words_in(self, total_vocabulary(los), total_vocabulary(ros)).iter().map(|h| {
            NewTodo {
                domain: "pairs".to_string(),
                other: *h
            }
        })).collect_vec()
    }

    fn get_friends_up_by_hop(&self, other: i64) -> Vec<NewTodo> {
        let p = self.get_pair(other);
        let los = &self.get_base_orthos_with_hops_overlapping(vec![p.first_word]);
        let ros = &self.get_base_orthos_with_hops_overlapping(vec![p.second_word]);
        los.iter().map(|o| {
            NewTodo {
                domain: "orthotopes".to_string(),
                other: ortho_to_orthotope(o).info_hash
            }
        }).chain(ros.iter().map(|o|{
            NewTodo {
                domain: "orthotopes".to_string(),
                other: ortho_to_orthotope(o).info_hash
            }
        })).chain(get_hashes_of_pairs_with_words_in(self, total_vocabulary(los), total_vocabulary(ros)).iter().map(|h| {
            NewTodo {
                domain: "pairs".to_string(),
                other: *h
            }
        })).collect_vec()
    }

    fn get_friends_up_by_contents(&self, other: i64) -> Vec<NewTodo> {
        let p = self.get_pair(other);
        let los = &&self.get_base_orthos_with_contents_overlapping(vec![p.first_word]);
        let ros = &self.get_base_orthos_with_contents_overlapping(vec![p.second_word]);
        los.iter().map(|o| {
            NewTodo {
                domain: "orthotopes".to_string(),
                other: ortho_to_orthotope(o).info_hash
            }
        }).chain(ros.iter().map(|o|{
            NewTodo {
                domain: "orthotopes".to_string(),
                other: ortho_to_orthotope(o).info_hash
            }
        })).chain(get_hashes_of_pairs_with_words_in(self, total_vocabulary(los), total_vocabulary(ros)).iter().map(|h| {
            NewTodo {
                domain: "pairs".to_string(),
                other: *h
            }
        })).collect_vec()
    }

    fn get_friends_up_forward(&self, other: i64) -> Vec<NewTodo> {
        let o = self.get_orthotope(other);

        if !o.is_base() {
            return vec![]
        }

        let dims = o.get_dims();

        let forwards = self.get_hashes_and_words_of_pairs_with_first_word(hashset! {o.get_origin()});
        let forward_pairs = forwards.iter().map(|x| x.2).collect_vec();
        let second_words: HashSet<Word> = forwards.iter().map(|x| x.1).collect();

        self.get_ortho_with_origin_in(second_words.clone()).iter().filter(|other_ortho| {
            other_ortho.get_dims() == dims
        }).map(|x| {
            NewTodo {
                domain: "orthotopes".to_string(),
                other: ortho_to_orthotope(x).info_hash
            }
        }).chain(forward_pairs.iter().map(|f| {
            NewTodo {
                domain: "pairs".to_string(),
                other: *f
            }
        })).collect_vec()
    }

    fn get_friends_up_back(&self, other: i64) -> Vec<NewTodo> {
        let o = self.get_orthotope(other);

        if !o.is_base() {
            return vec![]
        }

        let dims = o.get_dims();

        let backwards = self.get_hashes_and_words_of_pairs_with_second_word(hashset! {o.get_origin()});
        let backward_pairs = backwards.iter().map(|x| x.2).collect_vec();
        let second_words: HashSet<Word> = backwards.iter().map(|x| x.1).collect();

        self.get_ortho_with_origin_in(second_words.clone()).iter().filter(|other_ortho| {
            other_ortho.get_dims() == dims
        }).map(|x| {
            NewTodo {
                domain: "orthotopes".to_string(),
                other: ortho_to_orthotope(x).info_hash
            }
        }).chain(backward_pairs.iter().map(|f| {
            NewTodo {
                domain: "pairs".to_string(),
                other: *f
            }
        })).collect_vec()
    }

    fn get_friends_over_forward(&self, other: i64) -> Vec<NewTodo> {
        let old_orthotope = self.get_orthotope(other);
        let all_phrases = old_orthotope.origin_phrases();
        let all_second_words = all_phrases.iter().map(|p| p[1]).collect();
        let all_potential_orthos = self.get_ortho_with_origin_in(all_second_words);
    
        if all_potential_orthos.is_empty() {
            return vec![];
        }
    
        let all_phrase_heads: HashSet<i64> = old_orthotope
            .all_full_length_phrases()
            .iter()
            .map(|p| vec_of_words_to_big_int(p.to_vec()))
            .collect();
    
        let speculative_potential_phrases = self.get_phrase_hash_with_phrase_head_matching(all_phrase_heads);

        all_potential_orthos.iter().map(|x| {
            NewTodo {
                domain: "orthotopes".to_string(),
                other: ortho_to_orthotope(x).info_hash
            }
        }).chain(speculative_potential_phrases.iter().map(|f| {
            NewTodo {
                domain: "phrases".to_string(),
                other: *f
            }
        })).collect_vec()
    }

    fn get_friends_over_backward(&self, other: i64) -> Vec<NewTodo> {
        let old_orthotope = self.get_orthotope(other);

        let all_phrases = old_orthotope.origin_phrases();

        let firsts = all_phrases
            .iter()
            .map(|old_phrase| old_phrase[0])
            .collect::<HashSet<_>>();
        let backwards = self.get_words_of_pairs_with_second_word_in(firsts);
    
        let all_first_words = backwards.iter().map(|(f, _s)| f).copied().collect();
        let all_potential_orthos = self.get_ortho_with_origin_in(all_first_words);
    
        if all_potential_orthos.is_empty() {
            return vec![];
        }
    
        let all_phrase_tails: HashSet<i64> = old_orthotope
            .all_full_length_phrases()
            .iter()
            .map(|p| vec_of_words_to_big_int(p.to_vec()))
            .collect();
    
        let speculative_potential_phrases = self.get_phrase_hash_with_phrase_tail_matching(all_phrase_tails);

        all_potential_orthos.iter().map(|x| {
            NewTodo {
                domain: "orthotopes".to_string(),
                other: ortho_to_orthotope(x).info_hash
            }
        }).chain(speculative_potential_phrases.iter().map(|f| {
            NewTodo {
                domain: "phrases".to_string(),
                other: *f
            }
        })).collect_vec()
    }

    fn get_friends_over_by_origin(&self, other: i64) -> Vec<NewTodo> {
        let phrase = self.get_phrase(other);
        let lhs_phrase_head = &phrase[..phrase.len() - 1];
        let rhs_phrase_head = &phrase[1..];
        let head = phrase[0];
        let shift_left = phrase[1];
        let shift_right = phrase[2];
    
        let orthos_by_origin_left = self.get_orthos_with_origin(head);
        let lhs_by_origin = orthos_by_origin_left
            .into_iter()
            .filter(|o| o.origin_has_full_length_phrase(lhs_phrase_head));
        let orthos_by_origin_right = self.get_orthos_with_origin(shift_left);
    
        let rhs_by_origin = orthos_by_origin_right
            .into_iter()
            .filter(|o| o.origin_has_full_length_phrase(rhs_phrase_head));
    
        if lhs_by_origin.clone().next().is_none() || rhs_by_origin.clone().next().is_none() {
            return vec![];
        }
    
        let all_phrase_heads_left: HashSet<i64> = lhs_by_origin
            .clone()
            .flat_map(|o| {
                let phrases = o.phrases(shift_left);
                phrases
                    .iter()
                    .map(|p| vec_of_words_to_big_int(p.to_vec()))
                    .collect::<Vec<i64>>()
            })
            .collect();
    
        let all_phrase_heads_right: HashSet<i64> = rhs_by_origin
            .clone()
            .flat_map(|o| {
                let phrases = o.phrases(shift_right);
                phrases
                    .iter()
                    .map(|p| vec_of_words_to_big_int(p.to_vec()))
                    .collect::<Vec<i64>>()
            })
            .collect();
    
        let all_phrases =
            phrase_exists_db_filter(self, all_phrase_heads_left, all_phrase_heads_right);

            lhs_by_origin.map(|x| {
                NewTodo {
                    domain: "orthotopes".to_string(),
                    other: ortho_to_orthotope(&x).info_hash
                }
            }).chain(rhs_by_origin.map(|f| {
                NewTodo {
                    domain: "orthotopes".to_string(),
                    other: ortho_to_orthotope(&f).info_hash
                }
            })).chain(all_phrases.iter().map(|f| {
                NewTodo {
                    domain: "phrases".to_string(),
                    other: *f
                }
            })).collect_vec()
    }

    fn get_friends_over_by_hop(&self, other: i64) -> Vec<NewTodo> {
        let phrase = self.get_phrase(other);
        let lhs_phrase_head = &phrase[..phrase.len() - 1];
        let rhs_phrase_head = &phrase[1..];
    
        let orthos_by_hop_left = self.get_orthos_with_hops_overlapping(vec![phrase[0]]);
        let lhs_by_hop = orthos_by_hop_left
            .iter()
            .filter(|o| o.hop_has_full_length_phrase(lhs_phrase_head));
    
        let orthos_by_hop_right = self.get_orthos_with_hops_overlapping(vec![phrase[1]]);
        let rhs_by_hop = orthos_by_hop_right
            .iter()
            .filter(|o| o.hop_has_full_length_phrase(rhs_phrase_head));
    
        if lhs_by_hop.clone().next().is_none() || rhs_by_hop.clone().next().is_none() {
            return vec![];
        }
    
        let all_phrase_heads_left: HashSet<i64> = lhs_by_hop
            .clone()
            .flat_map(|o| {
                let axis = o.axis_of_change_between_names_for_hop(phrase[0], phrase[1]);
                let phrases = o.phrases(axis);
                phrases
                    .iter()
                    .map(|p| vec_of_words_to_big_int(p.to_vec()))
                    .collect::<Vec<_>>()
            })
            .collect();
    
        let all_phrase_heads_right: HashSet<i64> = rhs_by_hop
            .clone()
            .flat_map(|o| {
                let axis = o.axis_of_change_between_names_for_hop(phrase[1], phrase[2]);
                let phrases = o.phrases(axis);
                phrases
                    .iter()
                    .map(|p| vec_of_words_to_big_int(p.to_vec()))
                    .collect::<Vec<_>>()
            })
            .collect();
    
        let all_phrases =
            phrase_exists_db_filter(self, all_phrase_heads_left, all_phrase_heads_right);

            lhs_by_hop.map(|x| {
                NewTodo {
                    domain: "orthotopes".to_string(),
                    other: ortho_to_orthotope(&x).info_hash
                }
            }).chain(rhs_by_hop.map(|f| {
                NewTodo {
                    domain: "orthotopes".to_string(),
                    other: ortho_to_orthotope(&f).info_hash
                }
            })).chain(all_phrases.iter().map(|f| {
                NewTodo {
                    domain: "phrases".to_string(),
                    other: *f
                }
            })).collect_vec()
    }

    fn get_friends_over_by_contents(&self, other: i64) -> Vec<NewTodo> {
        let phrase = self.get_phrase(other);
        let lhs_phrase_head = &phrase[..phrase.len() - 1];
    let rhs_phrase_head = &phrase[1..];

    let orthos_by_contents_left = self.get_orthos_with_contents_overlapping(vec![phrase[0]]);
    let lhs_by_contents = orthos_by_contents_left
        .iter()
        .filter(|o| o.contents_has_full_length_phrase(lhs_phrase_head));

    let orthos_by_contents_right = self.get_orthos_with_contents_overlapping(vec![phrase[1]]);
    let rhs_by_contents = orthos_by_contents_right
        .iter()
        .filter(|o| o.contents_has_full_length_phrase(rhs_phrase_head));

    if lhs_by_contents.clone().next().is_none() || rhs_by_contents.clone().next().is_none() {
        return vec![];
    }

    let all_phrase_heads_left: HashSet<i64> = lhs_by_contents
        .clone()
        .flat_map(|o| {
            let axes = o.axes_of_change_between_names_for_contents(phrase[0], phrase[1]);
            axes.iter()
                .flat_map(|axis| {
                    let phrases = o.phrases(*axis);
                    phrases
                        .iter()
                        .map(|p| vec_of_words_to_big_int(p.to_vec()))
                        .collect::<Vec<_>>()
                })
                .collect_vec()
        })
        .collect();

    let all_phrase_heads_right: HashSet<i64> = rhs_by_contents
        .clone()
        .flat_map(|o| {
            let axes = o.axes_of_change_between_names_for_contents(phrase[1], phrase[2]);
            axes.iter()
                .flat_map(|axis| {
                    let phrases = o.phrases(*axis);
                    phrases
                        .iter()
                        .map(|p| vec_of_words_to_big_int(p.to_vec()))
                        .collect::<Vec<_>>()
                })
                .collect_vec()
        })
        .collect();

    let all_phrases =
        phrase_exists_db_filter(self, all_phrase_heads_left, all_phrase_heads_right);

        lhs_by_contents.map(|x| {
            NewTodo {
                domain: "orthotopes".to_string(),
                other: ortho_to_orthotope(&x).info_hash
            }
        }).chain(rhs_by_contents.map(|f| {
            NewTodo {
                domain: "orthotopes".to_string(),
                other: ortho_to_orthotope(&f).info_hash
            }
        })).chain(all_phrases.iter().map(|f| {
            NewTodo {
                domain: "phrases".to_string(),
                other: *f
            }
        })).collect_vec()
    }

    fn lhs_friends_share_provenance(&self, lhs_friends: &[NewTodo], rhs_friends: &[NewTodo]) -> bool {
        let lhs: HashSet<&NewTodo> = HashSet::from_iter(lhs_friends);
        lhs.is_superset(&HashSet::from_iter(rhs_friends))
    }

    fn rhs_friends_share_provenance(&self, lhs_friends: &[NewTodo], rhs_friends: &[NewTodo]) -> bool {
        let rhs: HashSet<&NewTodo> = HashSet::from_iter(rhs_friends);
        rhs.is_superset(&HashSet::from_iter(lhs_friends))
    }
}

pub fn get_hashes_of_pairs_with_words_in(
    holder: &Holder,
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
