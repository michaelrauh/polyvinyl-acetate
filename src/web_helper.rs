use std::{
    collections::{BTreeMap, HashSet},
    env,
};

use crate::{
    create_todo_entry, establish_connection_safe, get_relevant_vocabulary_reverse,
    models::{NewBook},
    ortho::Ortho,
    schema::{self, books, phrases},
    Book, NewTodo, Word,
};
use amiquip::{AmqpValue, FieldTable, QueueDeclareOptions};
use diesel::{PgConnection, QueryDsl, RunQueryDsl};

pub fn create_book(
    conn: &PgConnection,
    title: String,
    body: String,
) -> Result<Book, diesel::result::Error> {
    conn.build_transaction().serializable().run(|| {
        let book = create_book_entry(conn, title, body)?;
        let to_insert = vec![NewTodo {
            domain: "books".to_owned(),
            other: book.id,
        }];
        create_todo_entry(conn, to_insert)?;
        Ok(book)
    })
}

fn create_book_entry(
    conn: &PgConnection,
    title: String,
    body: String,
) -> Result<Book, diesel::result::Error> {
    diesel::insert_into(books::table)
        .values(&NewBook { title, body })
        .get_result(conn)
}

pub fn show_books() -> Result<String, anyhow::Error> {
    use crate::books;
    use crate::diesel::query_dsl::select_dsl::SelectDsl;
    let results: Vec<String> =
        SelectDsl::select(books, schema::books::title).load(&establish_connection_safe()?)?;

    Ok(results.join("\n"))
}

pub fn show_todos() -> Result<String, anyhow::Error> {
    use crate::schema::todos::dsl::todos;
    let results: i64 = todos.count().get_result(&establish_connection_safe()?)?;

    Ok(results.to_string())
}

pub fn count_sentences() -> Result<String, anyhow::Error> {
    use crate::schema::sentences::dsl::sentences;
    let results: i64 = sentences.count().get_result(&establish_connection_safe()?)?;

    Ok(results.to_string())
}

pub fn count_pairs() -> Result<String, anyhow::Error> {
    use crate::schema::pairs::dsl::pairs;
    let results: i64 = pairs.count().get_result(&establish_connection_safe()?)?;

    Ok(results.to_string())
}

pub fn splat_pairs() -> Result<String, anyhow::Error> {
    use crate::schema::pairs::dsl::pairs;
    let results: Vec<crate::models::Pair> = pairs
        .select(schema::pairs::all_columns)
        .get_results(&establish_connection_safe()?)?;
    let res: Vec<String> = results
        .iter()
        .map(|p| format!("{}, {}", p.first_word, p.second_word))
        .collect();

    Ok(res.join("\n"))
}

pub fn show_orthos(dims: BTreeMap<usize, usize>) -> Result<String, anyhow::Error> {
    let results = get_orthos_by_size(&establish_connection_safe()?, dims)?;

    let res = results.len().to_string();

    Ok(res)
}

pub fn splat_orthos(dims: BTreeMap<usize, usize>) -> Result<String, anyhow::Error> {
    let results = get_orthos_by_size(&establish_connection_safe()?, dims)?;

    let phrases: Vec<_> = results
        .iter()
        .map(|o| o.all_full_length_phrases())
        .collect();

    let all_words: HashSet<Word> = phrases.iter().flatten().flatten().cloned().collect();
    let mapping = get_relevant_vocabulary_reverse(&establish_connection_safe()?, all_words)?;

    let res = phrases
        .iter()
        .map(|o| {
            o.iter()
                .map(|s| {
                    s.iter()
                        .map(|w| mapping.get(w).expect("do not look up new words"))
                        .cloned()
                        .collect::<Vec<_>>()
                        .join(" ")
                })
                .collect::<Vec<_>>()
                .join("\n")
        })
        .collect::<Vec<_>>()
        .join("\n\n");

    Ok(res)
}

pub fn show_phrases() -> Result<String, anyhow::Error> {
    use crate::schema::phrases::dsl::phrases;
    let results: i64 = phrases.count().get_result(&establish_connection_safe()?)?;

    Ok(results.to_string())
}

fn get_orthos_by_size(
    conn: &PgConnection,
    dims: BTreeMap<usize, usize>,
) -> Result<Vec<Ortho>, anyhow::Error> {
    use crate::schema::orthotopes::table as orthotopes;
    let results: Vec<Vec<u8>> = orthotopes
        .select(schema::orthotopes::information)
        .load(conn)?;

    let actual: Vec<Ortho> = results
        .iter()
        .map(|x| bincode::deserialize(&x).expect("deserialization should succeed"))
        .filter(|o: &Ortho| o.get_dims() == dims)
        .collect();

    Ok(actual)
}

pub fn show_depth() -> Result<String, amiquip::Error> {
    use amiquip::Connection;

    let rabbit_url = env::var("RABBIT_URL").expect("RABBIT_URL must be set");

    let mut connection = Connection::insecure_open(&rabbit_url)?;

    let channel = connection.open_channel(None)?;
    let mut arguments = FieldTable::new();
    arguments.insert("x-max-priority".to_string(), AmqpValue::ShortInt(20));

    let queue = channel.queue_declare(
        "work",
        QueueDeclareOptions {
            durable: true,
            arguments,
            ..QueueDeclareOptions::default()
        },
    )?;

    let depth = queue
        .declared_message_count()
        .expect("queue must be declared non-immediate");
    Ok(depth.to_string())
}

pub fn parse_web_dims(web_dims_str: String) -> BTreeMap<usize, usize> {
    let nums: Vec<usize> = web_dims_str
        .split(',')
        .map(|s| s.trim())
        .filter(|s| !s.is_empty())
        .map(|s| s.parse().unwrap())
        .collect();

    let mut res = BTreeMap::default();
    for num in nums {
        *res.entry(num).or_insert(0) += 1
    }
    res
}

pub fn delete_db(conn: &PgConnection) -> Result<(), anyhow::Error> {
    use crate::books;
    use crate::pairs;
    use crate::schema::orthotopes::dsl::orthotopes;
    use crate::sentences::dsl::sentences;
    use crate::todos::dsl::todos;
    use crate::web_helper::phrases::dsl::phrases;

    diesel::delete(books).execute(conn)?;
    diesel::delete(todos).execute(conn)?;
    diesel::delete(sentences).execute(conn)?;
    diesel::delete(pairs).execute(conn)?;
    diesel::delete(orthotopes).execute(conn)?;
    diesel::delete(phrases).execute(conn)?;
    Ok(())
}
