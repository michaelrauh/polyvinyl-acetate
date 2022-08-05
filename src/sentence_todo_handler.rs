use std::collections::HashMap;

use crate::models::{NewPair, NewPhrase, Pair, Phrase, Todo};
use crate::{
    create_todo_entry, establish_connection_safe, get_relevant_vocabulary, ints_to_big_int,
    vec_of_words_to_big_int, NewTodo, Word,
};
use diesel::PgConnection;

pub fn handle_sentence_todo(todo: Todo) -> Result<(), anyhow::Error> {
    let conn = establish_connection_safe()?;
    conn.build_transaction().serializable().run(|| {
        let sentence = get_sentence(&conn, todo.other)?;
        let words = split_sentence(&sentence);
        let vocab = get_relevant_vocabulary(&conn, words.into_iter().collect())?;
        create_pairs(&conn, &sentence, &vocab)?;
        create_phrases(&conn, sentence, &vocab)?;
        Ok(())
    })
}

fn split_sentence(sentence: &str) -> Vec<String> {
    sentence
        .split_ascii_whitespace()
        .map(|x| x.to_string())
        .collect()
}

pub fn split_sentence_to_pairs(sentence: &str) -> Vec<(String, String)> {
    if sentence.len() < 2 {
        return vec![];
    }

    let words: Vec<String> = split_sentence(sentence);

    let mut shifted = words.iter();
    shifted.next().expect("there must be something here");
    std::iter::zip(words.iter(), shifted)
        .map(|(f, s)| (f.clone(), s.clone()))
        .collect()
}

fn create_pair_entry(
    conn: &PgConnection,
    to_insert: Vec<NewPair>,
) -> Result<Vec<Pair>, diesel::result::Error> {
    use crate::schema::pairs;
    use diesel::RunQueryDsl;
    diesel::insert_into(pairs::table)
        .values(&to_insert)
        .on_conflict_do_nothing()
        .get_results(conn)
}

fn create_phrases(
    conn: &PgConnection,
    sentence: String,
    vocab: &HashMap<String, i32>,
) -> Result<(), anyhow::Error> {
    let ps: Vec<Vec<String>> = split_sentence_to_phrases(sentence);
    let pi32s: Vec<Vec<Word>> = ps
        .iter()
        .map(|p| {
            p.iter()
                .map(|s| *vocab.get(s).expect("do not look up unknown words"))
                .collect()
        })
        .collect();
    let new_phrases: Vec<NewPhrase> = pi32s
        .into_iter()
        .filter(|phrase| phrase.len() > 2)
        .map(|v| NewPhrase {
            words: v.clone(),
            words_hash: vec_of_words_to_big_int(v.clone()),
            phrase_head: vec_of_words_to_big_int(v[..v.len() - 1].to_vec()),
            phrase_tail: vec_of_words_to_big_int(v[1..].to_vec()),
        })
        .collect();

    let phrases = create_phrase_entry(conn, new_phrases)?;
    let to_insert: Vec<NewTodo> = phrases
        .iter()
        .map(|p| NewTodo {
            domain: "phrases".to_owned(),
            other: p.id,
        })
        .collect();
    create_todo_entry(conn, to_insert)?;

    Ok(())
}

fn create_phrase_entry(
    conn: &PgConnection,
    to_insert: Vec<NewPhrase>,
) -> Result<Vec<Phrase>, diesel::result::Error> {
    use crate::schema::phrases;
    use diesel::RunQueryDsl;
    diesel::insert_into(phrases::table)
        .values(&to_insert)
        .on_conflict_do_nothing()
        .get_results(conn)
}

fn split_sentence_to_phrases(sentence: String) -> Vec<Vec<String>> {
    let words: Vec<String> = split_sentence(&sentence);

    heads(words)
        .iter()
        .flat_map(|ws| tails(ws.to_vec()))
        .collect()
}

fn heads(words: Vec<String>) -> Vec<Vec<String>> {
    let mut acc = vec![];
    for i in 1..words.len() + 1 {
        let sliced: Vec<String> = words[..i].to_vec();
        acc.push(sliced);
    }
    acc
}

fn tails(words: Vec<String>) -> Vec<Vec<String>> {
    let mut acc = vec![];
    for i in 0..words.len() {
        let sliced: Vec<String> = words[i..].to_vec();
        acc.push(sliced);
    }
    acc
}

fn create_pairs(
    conn: &PgConnection,
    sentence: &str,
    vocab: &HashMap<String, Word>,
) -> Result<(), anyhow::Error> {
    let tuples = split_sentence_to_pairs(sentence);
    let new_pairs: Vec<NewPair> = tuples
        .iter()
        .map(|(f, s)| {
            let first_number = *vocab
                .get(f)
                .expect("all words should be in the relevant vocab");
            let second_number = *vocab
                .get(s)
                .expect("all words should be in the relevant vocab");
            NewPair {
                first_word: first_number,
                second_word: second_number,
                pair_hash: ints_to_big_int(first_number, second_number),
            }
        })
        .collect();

    let pairs = create_pair_entry(conn, new_pairs)?;
    let to_insert: Vec<NewTodo> = pairs
        .iter()
        .map(|p| NewTodo {
            domain: "pairs".to_owned(),
            other: p.id,
        })
        .collect();
    create_todo_entry(conn, to_insert)?;

    Ok(())
}

fn get_sentence(conn: &PgConnection, pk: i32) -> Result<String, anyhow::Error> {
    use crate::schema::sentences::id;
    use crate::sentences::dsl::sentences;
    use diesel::{ExpressionMethods, QueryDsl, RunQueryDsl};
    let sentence: String = sentences
        .filter(id.eq(pk))
        .select(crate::sentences::sentence)
        .first(conn)?;

    Ok(sentence)
}

#[cfg(test)]
mod tests {

    use crate::sentence_todo_handler::{
        heads, split_sentence_to_pairs, split_sentence_to_phrases, tails,
    };

    #[test]
    fn it_splits_sentence_to_pairs_empty() {
        assert_eq!(split_sentence_to_pairs(""), vec![])
    }

    #[test]
    fn it_splits_sentence_to_pairs_one() {
        assert_eq!(split_sentence_to_pairs("one"), vec![])
    }

    #[test]
    fn it_splits_sentence_to_pairs_two() {
        assert_eq!(
            split_sentence_to_pairs("one two"),
            vec![("one".to_owned(), "two".to_owned())]
        )
    }

    #[test]
    fn it_splits_sentence_to_pairs_many() {
        assert_eq!(
            split_sentence_to_pairs("one two three four five"),
            vec![
                ("one".to_owned(), "two".to_owned()),
                ("two".to_owned(), "three".to_owned()),
                ("three".to_owned(), "four".to_owned()),
                ("four".to_owned(), "five".to_owned())
            ]
        )
    }

    #[test]
    fn it_finds_heads() {
        let ans: Vec<Vec<String>> =
            heads(vec!["one".to_owned(), "two".to_owned(), "three".to_owned()]);
        let expected: Vec<Vec<String>> = vec![
            vec!["one".to_owned()],
            vec!["one".to_owned(), "two".to_owned()],
            vec!["one".to_owned(), "two".to_owned(), "three".to_owned()],
        ];
        assert_eq!(ans, expected);
    }

    #[test]
    fn it_finds_tails() {
        let ans: Vec<Vec<String>> =
            tails(vec!["one".to_owned(), "two".to_owned(), "three".to_owned()]);
        let expected: Vec<Vec<String>> = vec![
            vec!["one".to_owned(), "two".to_owned(), "three".to_owned()],
            vec!["two".to_owned(), "three".to_owned()],
            vec!["three".to_owned()],
        ];
        assert_eq!(ans, expected);
    }

    #[test]
    fn it_splits_sentence_to_phrases_empty() {
        let ans: Vec<Vec<String>> = split_sentence_to_phrases("".to_owned());
        let expected: Vec<Vec<String>> = vec![];
        assert_eq!(ans, expected);
    }

    #[test]
    fn it_splits_sentence_to_phrases_one() {
        let ans: Vec<Vec<String>> = split_sentence_to_phrases("one".to_owned());
        let expected: Vec<Vec<String>> = vec![vec!["one".to_owned()]];
        assert_eq!(ans, expected);
    }

    #[test]
    fn it_splits_sentence_to_phrases_many() {
        let ans: Vec<Vec<String>> = split_sentence_to_phrases("one two three".to_owned());
        let expected: Vec<Vec<String>> = vec![
            vec!["one".to_owned()],
            vec!["one".to_owned(), "two".to_owned()],
            vec!["two".to_owned()],
            vec!["one".to_owned(), "two".to_owned(), "three".to_owned()],
            vec!["two".to_owned(), "three".to_owned()],
            vec!["three".to_owned()],
        ];
        assert_eq!(ans, expected);
    }
}
