use std::{collections::BTreeMap, env};

use crate::{
    create_todo_entry, establish_connection,
    models::{NewBook, Orthotope},
    ortho::Ortho,
    schema::{self, books},
    Book, NewTodo,
};
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
        create_todo_entry(conn, &to_insert)?;
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

pub fn show_books() -> Result<String, diesel::result::Error> {
    use crate::books;
    use crate::diesel::query_dsl::select_dsl::SelectDsl;
    let results: Vec<String> =
        SelectDsl::select(books, schema::books::title).load(&establish_connection())?;

    Ok(results.join("\n"))
}

pub fn show_todos() -> Result<String, diesel::result::Error> {
    use crate::schema::todos::dsl::todos;
    let results: i64 = todos.count().get_result(&establish_connection())?;

    Ok(results.to_string())
}

pub fn count_sentences() -> Result<String, diesel::result::Error> {
    use crate::schema::sentences::dsl::sentences;
    let results: i64 = sentences.count().get_result(&establish_connection())?;

    Ok(results.to_string())
}

pub fn count_pairs() -> Result<String, diesel::result::Error> {
    use crate::schema::pairs::dsl::pairs;
    let results: i64 = pairs.count().get_result(&establish_connection())?;

    Ok(results.to_string())
}

pub fn show_orthos(dims: BTreeMap<usize, usize>) -> Result<String, anyhow::Error> {
    let results = get_orthos_by_size(&establish_connection(), dims)?;

    let res = results.len().to_string();

    Ok(res)
}

fn get_orthos_by_size(
    conn: &PgConnection,
    dims: BTreeMap<usize, usize>,
) -> Result<Vec<Ortho>, anyhow::Error> {
    use crate::schema::orthotopes::table as orthotopes;
    let results: Vec<Orthotope> = orthotopes
        .select(schema::orthotopes::all_columns)
        .load(conn)?;

    let res: Vec<Ortho> = results
        .iter()
        .map(|x| bincode::deserialize(&x.information).expect("deserialization should succeed"))
        .collect();

    let actual = res.into_iter().filter(|o| o.get_dims() == dims).collect();

    Ok(actual)
}

pub fn show_depth() -> Result<String, amiquip::Error> {
    use amiquip::Connection;

    let rabbit_url = env::var("RABBIT_URL").expect("RABBIT_URL must be set");

    let mut connection = Connection::insecure_open(&rabbit_url)?;

    let channel = connection.open_channel(None)?;
    let queue = channel.queue_declare(
        "work",
        amiquip::QueueDeclareOptions {
            durable: true,
            ..amiquip::QueueDeclareOptions::default()
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
    use crate::todos::dsl::todos;
    use crate::sentences::dsl::sentences;
    use crate::pairs;
    use crate::schema::orthotopes::dsl::orthotopes;
        
    let effa = diesel::delete(books).execute(conn)?;
    println!("books: {:?}", effa);
    let effb = diesel::delete(todos).execute(conn)?;
    println!("todos: {:?}", effb);
    let effc = diesel::delete(sentences).execute(conn)?;
    println!("sentences: {:?}", effc);
    let effd = diesel::delete(pairs).execute(conn)?;
    println!("pairs: {:?}", effd);
    let effe = diesel::delete(orthotopes).execute(conn)?;
    println!("orthotopes: {:?}", effe);
    Ok(())
}
