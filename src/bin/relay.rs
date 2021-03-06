use std::{env, thread::sleep};

use amiquip::{AmqpProperties, Exchange, Publish, QueueDeclareOptions};
use diesel::{query_dsl::methods::FilterDsl, RunQueryDsl};
use polyvinyl_acetate::{
    establish_connection,
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
        sleep(core::time::Duration::from_secs(1))
    }
}

pub fn get_todos() -> Result<Vec<Todo>, diesel::result::Error> {
    use polyvinyl_acetate::schema::todos::dsl::todos;
    let results = todos.load(&establish_connection())?;

    Ok(results)
}

pub fn delete_todos(todos_to_delete: Vec<Todo>) -> Result<usize, diesel::result::Error> {
    use crate::todos::dsl::todos;
    use diesel::ExpressionMethods;
    let ids = todos_to_delete.iter().map(|t| t.id);
    let f = todos.filter(schema::todos::id.eq_any(ids));
    diesel::delete(f).execute(&establish_connection())
}

fn apply() -> Result<usize, anyhow::Error> {
    establish_connection()
        .build_transaction()
        .serializable()
        .run(|| {
            let todos = get_todos()?;
            let number_published = publish(&todos)?;
            delete_todos(todos)?;
            Ok(number_published)
        })
}

fn publish(todos: &[Todo]) -> Result<usize, amiquip::Error> {
    use amiquip::Connection;

    let rabbit_url = env::var("RABBIT_URL").expect("RABBIT_URL must be set");

    let mut connection = Connection::insecure_open(&rabbit_url)?;

    let channel = connection.open_channel(None)?;

    let _ = channel.queue_declare(
        "work",
        QueueDeclareOptions {
            durable: true,
            ..QueueDeclareOptions::default()
        },
    )?;

    let exchange = Exchange::direct(&channel);

    for todo in todos {
        let data = bincode::serialize(&todo).expect("bincode should be able to serialize");
        exchange.publish(Publish::with_properties(
            &data,
            "work",
            AmqpProperties::default().with_delivery_mode(2),
        ))?;
    }

    connection.close()?;
    Ok(todos.len())
}
