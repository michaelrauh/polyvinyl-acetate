extern crate openssl;

#[allow(unused_imports)]
#[macro_use]
extern crate diesel;

use polyvinyl_acetate::{create_book, establish_connection, show_books};

#[macro_use]
extern crate rocket;

#[macro_use]
extern crate diesel_migrations;

use diesel_migrations::embed_migrations;
use rocket::response::status::Conflict;
use rocket::serde::json::Json;
use serde::Deserialize;

embed_migrations!("./migrations");

#[get("/")]
fn index() -> Result<String, Conflict<String>> {
    show_books().map_err(|e| { rocket::response::status::Conflict(Some(e.to_string())) })
}

#[derive(Deserialize)]
struct WebBook {
    title: String,
    body: String,
}

#[post("/add", format = "json", data = "<web_book>")]
fn add(web_book: Json<WebBook>) -> Result<String, Conflict<String>> {
    match create_book(
        &establish_connection(),
        web_book.title.clone(),
        web_book.body.clone(),
    ) {
        Ok(book) => Ok(book.title),
        Err(error) => Err(rocket::response::status::Conflict(Some(error.to_string()))),
    }
}

#[launch]
fn rocket() -> _ {
    let connection = establish_connection();
    embedded_migrations::run_with_output(&connection, &mut std::io::stdout()).unwrap();

    rocket::build().mount("/", routes![index, add])
}
