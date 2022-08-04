use std::env;

use amiquip::{AmqpProperties, AmqpValue, Exchange, FieldTable, Publish, QueueDeclareOptions};
use diesel::{query_dsl::methods::FilterDsl, PgConnection, RunQueryDsl};
use polyvinyl_acetate::{
    establish_connection_safe,
    models::Todo,
    schema::{self, todos},
};

fn main() {
    loop {
        match apply() {
            Ok(amount) => {
                if amount > 0 {
                    println!("successfully relayed {} messages", amount)
                }
            }
            Err(e) => println!("failure: {}", e),
        }
    }
}

pub fn get_todos(conn: &PgConnection) -> Result<Vec<Todo>, diesel::result::Error> {
    use polyvinyl_acetate::schema::todos::dsl::todos;
    let results = diesel::QueryDsl::limit(todos, 1000).load(conn)?;

    Ok(results)
}

pub fn delete_todos(
    conn: &PgConnection,
    todos_to_delete: Vec<Todo>,
) -> Result<usize, diesel::result::Error> {
    use crate::todos::dsl::todos;
    use diesel::ExpressionMethods;
    let ids = todos_to_delete.iter().map(|t| t.id);
    let f = todos.filter(schema::todos::id.eq_any(ids));
    diesel::delete(f).execute(conn)
}

fn apply() -> Result<usize, anyhow::Error> {
    let conn = establish_connection_safe()?;
    conn.build_transaction().serializable().run(|| {
        let todos = get_todos(&conn)?;
        let number_published = publish(&todos)?;
        delete_todos(&conn, todos)?;
        Ok(number_published)
    })
}

fn publish(todos: &[Todo]) -> Result<usize, amiquip::Error> {
    use amiquip::Connection;

    let rabbit_url = env::var("RABBIT_URL").expect("RABBIT_URL must be set");

    let mut connection = Connection::insecure_open(&rabbit_url)?;

    let channel = connection.open_channel(None)?;

    let mut arguments = FieldTable::new();
    arguments.insert("x-max-priority".to_string(), AmqpValue::ShortInt(20));

    let _ = channel.queue_declare(
        "work",
        QueueDeclareOptions {
            durable: true,
            arguments,
            ..QueueDeclareOptions::default()
        },
    )?;

    let exchange = Exchange::direct(&channel);

    for todo in todos {
        let data = bincode::serialize(&todo).expect("bincode should be able to serialize");
        exchange.publish(Publish::with_properties(
            &data,
            "work",
            AmqpProperties::default()
                .with_delivery_mode(2)
                .with_priority(domain_to_priority(&todo.domain)),
        ))?;
    }

    connection.close()?;
    Ok(todos.len())
}

fn domain_to_priority(domain: &str) -> u8 {
    match domain {
        "books" => 1,
        "sentences" => 2,
        "pairs" => 3,
        "pair_up" => 4,
        "ex_nihilo_ffbb" => 5,
        "ex_nihilo_fbbf" => 6,
        "up_by_origin" => 7,
        "up_by_hop" => 8,
        "up_by_contents" => 9,
        "phrases" => 10,
        "phrase_by_origin" => 11,
        "phrase_by_hop" => 12,
        "phrase_by_contents" => 13,
        "orthotopes" => 14,
        "ortho_up" => 15,
        "ortho_up_forward" => 16,
        "ortho_up_back" => 17,
        "ortho_over" => 18,
        "ortho_over_forward" => 19,
        "ortho_over_back" => 20,
        other => {
            panic!("getting unexpected todo with domain: {other}")
        }
    }
}
