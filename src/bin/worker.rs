use amiquip::{
    AmqpValue, Connection, ConsumerMessage, ConsumerOptions, FieldTable, QueueDeclareOptions,
    Result,
};
use polyvinyl_acetate::models::Todo;
use polyvinyl_acetate::worker_helper;
use std::{env};

use opentelemetry::global;
use tracing_subscriber::{
    fmt, layer::SubscriberExt, util::SubscriberInitExt,
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
    
    
    // Allows you to pass along context (i.e., trace IDs) across services
global::set_text_map_propagator(opentelemetry_jaeger::Propagator::new());
// Sets up the machinery needed to export data to Jaeger
// There are other OTel crates that provide pipelines for the vendors
// mentioned earlier.
let tracer = opentelemetry_jaeger::new_pipeline()
.with_agent_endpoint("host.docker.internal:6831")
    .with_service_name("pvac")
    .install_simple().expect("tracer made");

// Create a tracing layer with the configured tracer
let opentelemetry = tracing_opentelemetry::layer().with_tracer(tracer);

// The SubscriberExt and SubscriberInitExt traits are needed to extend the
// Registry to accept `opentelemetry (the OpenTelemetryLayer type).
tracing_subscriber::registry()
    .with(opentelemetry)
    // Continue logging to stdout
    .with(fmt::Layer::default())
    .try_init().expect("subscribed");

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
