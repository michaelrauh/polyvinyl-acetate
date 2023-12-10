use amiquip::{
    AmqpValue, Connection, ConsumerMessage, ConsumerOptions, FieldTable, QueueDeclareOptions,
    Result,
};
use polyvinyl_acetate::worker_helper;
use polyvinyl_acetate::{models::Todo, Holder};
use std::env;

use diesel::{
    r2d2::{ConnectionManager, Pool},
    PgConnection,
};

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

    // global::set_text_map_propagator(opentelemetry_jaeger::Propagator::new());
    // let tracer = opentelemetry_jaeger::new_pipeline()
    //     .with_service_name("pvac")
    //     .install_simple()
    //     .expect("tracer made");

    // let opentelemetry = tracing_opentelemetry::layer().with_tracer(tracer);

    // tracing_subscriber::registry()
    //     .with(opentelemetry)
    //     // .with(fmt::Layer::default())
    //     .try_init()
    //     .expect("subscribed");

    let database_url = env::var("DATABASE_URL").expect("DATABASE_URL must be set");

    let manager = ConnectionManager::<PgConnection>::new(&database_url);
    let pool = Pool::builder()
        .max_size(1)
        .build(manager)
        .expect("Failed to create pool.");
    let mut holder = Holder::new();

    for (i, message) in consumer.receiver().iter().enumerate() {
        println!("number of messages: {}", i);
        match message {
            ConsumerMessage::Delivery(delivery) => {
                let todo: Todo = bincode::deserialize(&delivery.body)?;
                println!("todo: {:?}", &todo);
                match worker_helper::handle_todo(todo, pool.clone(), &mut holder) {
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
