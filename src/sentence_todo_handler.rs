use diesel::PgConnection;
use crate::{create_todo_entry, establish_connection, NewTodo, string_to_signed_int};
use crate::models::{NewPair, Pair, Sentence, Todo};

pub fn handle_sentence_todo(todo: Todo) -> Result<(), anyhow::Error> {
    let conn = establish_connection();
    conn.build_transaction().serializable().run(|| {
        let sentence = get_sentence(&conn, todo.other)?;
        create_pairs(&conn, sentence.sentence)?;
        Ok(())
    })
}

pub fn split_sentence_to_pairs(sentence: String) -> Vec<(String, String)> {
    if sentence.len() < 2 {
        return vec![];
    }

    let words: Vec<String> = sentence
        .split_ascii_whitespace()
        .map(|x| x.to_string())
        .collect();

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
    use diesel::RunQueryDsl;
    use crate::schema::pairs;
    diesel::insert_into(pairs::table)
        .values(&to_insert)
        .on_conflict_do_nothing()
        .get_results(conn)
}


fn create_pairs(conn: &PgConnection, sentence: String) -> Result<(), anyhow::Error> {
    let tuples = split_sentence_to_pairs(sentence);
    let new_pairs: Vec<NewPair> = tuples
        .iter()
        .map(|(f, s)| NewPair {
            first_word: f.clone(),
            second_word: s.clone(),
            first_word_hash: string_to_signed_int(f),
            second_word_hash: string_to_signed_int(s),
        })
        .collect();

    let pairs = create_pair_entry(conn, new_pairs)?;
    let to_insert = pairs
        .iter()
        .map(|p| NewTodo {
            domain: "pairs".to_owned(),
            other: p.id,
        })
        .collect();
    create_todo_entry(conn, &to_insert)?;

    Ok(())
}

fn get_sentence(conn: &PgConnection, pk: i32) -> Result<Sentence, anyhow::Error> {
    use diesel::{ExpressionMethods, QueryDsl, RunQueryDsl};
    use crate::schema::sentences::id;
    use crate::sentences::dsl::sentences;
    let sentence: Sentence = sentences
        .filter(id.eq(pk))
        .select(crate::sentences::all_columns)
        .first(conn)?;

    Ok(sentence)
}

#[cfg(test)]
mod tests {

    use crate::sentence_todo_handler::split_sentence_to_pairs;

    #[test]
    fn it_splits_sentence_to_pairs_empty() {
        assert_eq!(split_sentence_to_pairs("".to_string()), vec![])
    }

    #[test]
    fn it_splits_sentence_to_pairs_one() {
        assert_eq!(split_sentence_to_pairs("one".to_string()), vec![])
    }

    #[test]
    fn it_splits_sentence_to_pairs_two() {
        assert_eq!(
            split_sentence_to_pairs("one two".to_string()),
            vec![("one".to_owned(), "two".to_owned())]
        )
    }

    #[test]
    fn it_splits_sentence_to_pairs_many() {
        assert_eq!(
            split_sentence_to_pairs("one two three four five".to_string()),
            vec![
                ("one".to_owned(), "two".to_owned()),
                ("two".to_owned(), "three".to_owned()),
                ("three".to_owned(), "four".to_owned()),
                ("four".to_owned(), "five".to_owned())
            ]
        )
    }
}
