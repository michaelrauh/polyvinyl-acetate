use amiquip::{Connection, ConsumerMessage, ConsumerOptions, QueueDeclareOptions, Result};
use polyvinyl_acetate::models::Todo;
use std::env;
use polyvinyl_acetate::worker_helper;

fn main() {
    get().expect("Rabbit should not err");
}

fn get() -> Result<(), anyhow::Error> {
    let rabbit_url = env::var("RABBIT_URL").expect("RABBIT_URL must be set");

    let mut connection = Connection::insecure_open(&rabbit_url)?;

    let channel = connection.open_channel(None)?;

    let queue = channel.queue_declare(
        "work",
        QueueDeclareOptions {
            durable: true,
            ..QueueDeclareOptions::default()
        },
    )?;

    channel.qos(0, 1, false)?;

    let consumer = queue.consume(ConsumerOptions::default())?;
    println!("Waiting for messages");

    for (i, message) in consumer.receiver().iter().enumerate() {
        println!("number of messages: {}", i);
        match message {
            ConsumerMessage::Delivery(delivery) => {
                let todo: Todo = bincode::deserialize(&delivery.body)?;
                println!("todo: {:?}", &todo);
                match worker_helper::handle_todo(todo) {
                    Ok(_) => consumer.ack(delivery)?,
                    Err(e) => {
                        println!("requeuing because of {e}");
                        consumer.nack(delivery, true)?
                    }
                }
            }
            other => {
                println!("Consumer ended: {:?}", other);
                break;
            }
        }
    }

    connection.close()?;
    Ok(())
}