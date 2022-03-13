use std::env;


use polyvinyl_acetate::{show_posts, create_post, establish_connection};
use rand::{Rng, thread_rng, distributions::Alphanumeric};


use amiquip::{Connection};

extern crate openssl;
extern crate diesel;

#[macro_use] extern crate rocket;

#[macro_use]
extern crate diesel_migrations;

use diesel_migrations::embed_migrations;
embed_migrations!("./migrations");

#[get("/")]
fn index() -> String {
    show_posts()
}

#[post("/add")]
fn add() -> &'static str {
    let title: String = thread_rng()
    .sample_iter(&Alphanumeric)
    .take(30)
    .map(char::from)
    .collect();
    
    create_post(&establish_connection(), &title, "body");
    "OK\n"
}

#[launch]
fn rocket() -> _ {
    let connection = establish_connection();
    embedded_migrations::run_with_output(&connection, &mut std::io::stdout()).unwrap();

    let rabbit_url = env::var("RABBIT_URL")
        .expect("RABBIT_URL must be set");
    println!("rabbit url is: {}", rabbit_url);

    let mut connection = Connection::insecure_open(&rabbit_url).unwrap();
   
    rocket::build().mount("/", routes![index, add])   
}