use std::env;
use amiquip::{Connection, ConsumerMessage, ConsumerOptions, QueueDeclareOptions, Result};

fn main() {
    get().unwrap();
}

fn get() -> Result<(), amiquip::Error> {
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
                println!("body: {:?}", &delivery.body);

                consumer.ack(delivery)?;
            }
            other => {
                println!("Consumer ended: {:?}", other);
                break;
            }
        }
    }

    connection.close()
}