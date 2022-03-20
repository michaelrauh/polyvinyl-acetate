use std::{env, thread::sleep};

use amiquip::{AmqpProperties, Exchange, Publish, QueueDeclareOptions};
use polyvinyl_acetate::{delete_todos, establish_connection, get_todos, models::Todo};

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
