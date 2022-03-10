use polyvinyl_acetate::{show_posts, create_post, establish_connection};
use rand::{Rng, thread_rng, distributions::Alphanumeric};

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
    .map(char::from)
    .collect();
    
    create_post(&establish_connection(), &title, "body");
    "OK"
}

#[launch]
fn rocket() -> _ {
    let connection = establish_connection();
    embedded_migrations::run_with_output(&connection, &mut std::io::stdout()).unwrap();
    rocket::build().mount("/", routes![index, add])   
}