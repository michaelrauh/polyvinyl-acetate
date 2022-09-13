use amiquip::{
    AmqpValue, Connection, ConsumerMessage, ConsumerOptions, FieldTable, QueueDeclareOptions,
    Result,
};
use polyvinyl_acetate::models::Todo;
use polyvinyl_acetate::worker_helper;
use std::{env, io};
use tracing::{span_enabled, Level, trace_span};

fn main() {
    get().expect("Rabbit should not err");
}

fn get() -> Result<(), anyhow::Error> {
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

    channel.qos(0, 1, false)?;

    let consumer = queue.consume(ConsumerOptions::default())?;
    let subscriber = tracing_subscriber::FmtSubscriber::builder()
        .with_max_level(Level::TRACE)
        .finish();
    tracing::subscriber::set_global_default(subscriber).expect("setting default subscriber failed");
    let number_of_teams: i32 = 3;
    tracing::info!(number_of_teams, "We've got {} teams!", number_of_teams);

    for (i, message) in consumer.receiver().iter().enumerate() {
        println!("number of messages: {}", i);
        // tracing::info!(number_of_teams, "We've got {} teams!", number_of_teams);
        let _ = tracing::info_span!("here");
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
