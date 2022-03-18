extern crate openssl;

#[allow(unused_imports)]
#[macro_use]
extern crate diesel;

use polyvinyl_acetate::{create_book, establish_connection, show_books, show_todos, show_depth};

#[macro_use]
extern crate rocket;

#[macro_use]
extern crate diesel_migrations;

use diesel_migrations::embed_migrations;
use rocket::response::status::Conflict;
use rocket::routes;
use rocket::serde::json::Json;
use serde::Deserialize;

embed_migrations!("./migrations");

#[get("/")]
fn index() -> Result<String, Conflict<String>> {
    show_books().map_err(|e| Conflict(Some(e.to_string())))
}

#[get("/count")]
fn count() -> Result<String, Conflict<String>> {
    show_todos().map_err(|e| Conflict(Some(e.to_string())))
}

#[get("/depth")]
fn depth() -> Result<String, Conflict<String>> {
    show_depth().map_err(|e| Conflict(Some(e.to_string())))
}

#[derive(Deserialize)]
struct WebBook {
    title: String,
    body: String,
}

#[post("/add", format = "json", data = "<web_book>")]
fn add(web_book: Json<WebBook>) -> Result<String, Conflict<String>> {
    let book = create_book(
        &establish_connection(),
        web_book.title.clone(),
        web_book.body.clone(),
    )
    .map_err(|error| Conflict(Some(error.to_string())))?;

    Ok(book.title)
}

#[launch]
fn rocket() -> _ {
    let connection = establish_connection();
    embedded_migrations::run_with_output(&connection, &mut std::io::stdout()).unwrap();

    rocket::build().mount("/", routes![index, add, count, depth])
}
