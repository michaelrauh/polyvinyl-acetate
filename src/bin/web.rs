use polyvinyl_acetate::{show_posts, create_post, establish_connection};
use rand::{Rng, thread_rng, distributions::Alphanumeric};

#[macro_use] extern crate rocket;

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
    "OK"
}

#[launch]
fn rocket() -> _ {
    rocket::build().mount("/", routes![index, add])   
}