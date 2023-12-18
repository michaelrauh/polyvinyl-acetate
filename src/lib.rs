pub mod models;

use itertools::Itertools;
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

use models::{Book, NewPair, NewTodo};
use std::collections::{HashMap, HashSet};
use std::{
    collections::hash_map::DefaultHasher,
    hash::{Hash, Hasher},
};

type Word = i32;

#[derive(Debug)]
pub struct Holder {
    books: sled::Db,
    vocabulary: sled::Db,
    sentences: HashMap<i64, String>,
    todos: HashMap<String, HashSet<i64>>,
    pairs_by_first: HashMap<Word, HashSet<NewPair>>,
    pairs_by_second: HashMap<Word, HashSet<NewPair>>,
    pairs_by_hash: HashMap<i64, NewPair>,
    phrases_by_head: HashMap<i64, HashSet<i64>>,
    phrases_by_tail: HashMap<i64, HashSet<i64>>,
    phrases_by_hash: HashMap<i64, Vec<Word>>,
    orthos_by_hash: sled::Db,
    orthos_by_hop: HashMap<Word, HashSet<NewOrthotope>>,
    orthos_by_contents: HashMap<Word, HashSet<NewOrthotope>>,
    orthos_by_origin: HashMap<Word, HashSet<Ortho>>,
}

impl Holder {
    pub fn new() -> Self {
        Holder {
            books: sled::open("db/books.sled").unwrap(),
            vocabulary: sled::open("db/vocab.sled").unwrap(),
            sentences: HashMap::default(),
            todos: HashMap::default(),
            pairs_by_first: HashMap::default(),
            pairs_by_second: HashMap::default(),
            pairs_by_hash: HashMap::default(),
            phrases_by_head: HashMap::default(),
            phrases_by_tail: HashMap::default(),
            phrases_by_hash: HashMap::default(),
            orthos_by_hash: sled::open("db/obh.sled").unwrap(),
            orthos_by_hop: HashMap::default(),
            orthos_by_contents: HashMap::default(),
            orthos_by_origin: HashMap::default(),
        }
    }

    pub fn get_stats(&self) {
        dbg!(&self.todos.iter().map(|(_k, v)| { v.len() }).sum::<usize>());
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

    fn get_vocabulary_slice_with_words(&self, firsts: HashSet<Word>) -> HashMap<Word, String> {
        self.vocabulary
            .into_iter()
            .filter(|x| {
                let word: Word = bincode::deserialize(&x.clone().unwrap().0).unwrap();
                firsts.contains(&word)
            })
            .map(|x| {
                let word: Word = bincode::deserialize(&x.clone().unwrap().0).unwrap();
                let val: String = bincode::deserialize(&x.unwrap().1).unwrap();
                (word, val)
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

    fn get_book(&self, pk: i64) -> Book {
        let book = self
            .books
            .get(bincode::serialize(&pk).unwrap())
            .unwrap()
            .unwrap();
        bincode::deserialize(&book).unwrap()
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
        let current: HashSet<String> = self
            .vocabulary
            .into_iter()
            .keys()
            .map(|x| {
                let foo = x.unwrap();
                bincode::deserialize(&foo).unwrap()
            })
            .collect::<HashSet<_>>();
        let words_to_insert: HashSet<String> = to_insert.iter().map(|w| w.word.clone()).collect();
        let new = words_to_insert.difference(&current).collect_vec();
        let current_index = self.vocabulary.len();
        let new_indices = current_index..(current_index + new.len());

        words_to_insert.iter().zip(new_indices).for_each(|(k, v)| {
            let to_insert: Word = v.try_into().unwrap();
            self.vocabulary
                .insert(
                    bincode::serialize(k).unwrap(),
                    bincode::serialize(&to_insert).unwrap(),
                )
                .unwrap();
        });
    }

    fn get_vocabulary(&self, words: HashSet<String>) -> HashMap<String, Word> {
        self.vocabulary
            .iter()
            .filter(|x| {
                let word: String = bincode::deserialize(&x.clone().unwrap().0).unwrap();
                words.contains(&word)
            })
            .map(|x| {
                let word: String = bincode::deserialize(&x.clone().unwrap().0).unwrap();
                let val: Word = bincode::deserialize(&x.unwrap().1).unwrap();
                (word, val)
            })
            .collect()
    }

    fn get_pair(&self, key: i64) -> NewPair {
        self.pairs_by_hash[&key].clone()
    }

    fn get_phrase(&self, key: i64) -> Vec<Word> {
        self.phrases_by_hash[&key].to_owned()
    }

    fn get_orthotope(&self, key: i64) -> Ortho {
        let ortho = self
            .orthos_by_hash
            .get(bincode::serialize(&key).unwrap())
            .unwrap()
            .unwrap();
        bincode::deserialize(&ortho).unwrap()
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

    pub fn insert_todos(&mut self, domain: &str, hashes: Vec<i64>) {
        let todos = self
            .todos
            .entry(domain.to_owned())
            .or_insert(HashSet::default());
        hashes.iter().for_each(|h| {
            todos.insert(*h);
        });
    }

    fn get_sentence(&self, pk: i64) -> String {
        self.sentences[&pk].clone()
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
                .insert(
                    bincode::serialize(&new_ortho.info_hash).unwrap(),
                    bincode::serialize(&new_ortho.information.clone()).unwrap(),
                )
                .unwrap();
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
        let b = Book {
            id: string_to_signed_int(&title).try_into().unwrap(),
            title,
            body,
        };
        self.books
            .insert(
                bincode::serialize(&b.id).unwrap(),
                bincode::serialize(&b.clone()).unwrap(),
            )
            .unwrap();
        b
    }

    pub fn get_next_todo(&mut self) -> Option<NewTodo> {
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

        let _bfs = vec![
            "books",
            "sentences",
            "pairs",
            "phrases",
            "ex_nihilo_ffbb",
            "ex_nihilo_fbbf",
            "pair_up",
            "up_by_origin",
            "up_by_hop",
            "up_by_contents",
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

        for domain in dfs {
            if self.todos.get_mut(domain).is_some() {
                let res_set: &mut HashSet<i64> = &mut self.todos.get_mut(domain).unwrap();
                if res_set.iter().next().is_some() {
                    let idx = res_set.iter().next().unwrap().clone();
                    res_set.remove(&idx);

                    return Some(NewTodo {
                        domain: domain.to_owned(),
                        other: idx,
                    });
                }
            }
        }
        None
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
