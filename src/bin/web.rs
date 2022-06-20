extern crate openssl;

#[allow(unused_imports)]
#[macro_use]
extern crate diesel;

use polyvinyl_acetate::web_helper::{
    count_pairs, count_sentences, create_book, show_books, show_depth, show_orthos, show_phrases,
    show_todos, splat_orthos,
};
use polyvinyl_acetate::{establish_connection, web_helper};

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

#[get("/sentences")]
fn sentences() -> Result<String, Conflict<String>> {
    count_sentences().map_err(|e| Conflict(Some(e.to_string())))
}

#[get("/pairs")]
fn pairs() -> Result<String, Conflict<String>> {
    count_pairs().map_err(|e| Conflict(Some(e.to_string())))
}

#[get("/count")]
fn count() -> Result<String, Conflict<String>> {
    show_todos().map_err(|e| Conflict(Some(e.to_string())))
}

#[get("/depth")]
fn depth() -> Result<String, Conflict<String>> {
    show_depth().map_err(|e| Conflict(Some(e.to_string())))
}

#[get("/phrases")]
fn phrases() -> Result<String, Conflict<String>> {
    show_phrases().map_err(|e| Conflict(Some(e.to_string())))
}

#[get("/orthos?<dims>")]
fn orthos(dims: String) -> Result<String, Conflict<String>> {
    show_orthos(web_helper::parse_web_dims(dims)).map_err(|e| Conflict(Some(e.to_string())))
}

#[get("/splat?<dims>")]
fn splat(dims: String) -> Result<String, Conflict<String>> {
    splat_orthos(web_helper::parse_web_dims(dims)).map_err(|e| Conflict(Some(e.to_string())))
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

#[delete("/")]
fn delete() -> Result<(), Conflict<String>> {
    web_helper::delete_db(&establish_connection())
        .map_err(|error| Conflict(Some(error.to_string())))?;
    Ok(())
}

#[launch]
fn rocket() -> _ {
    embedded_migrations::run_with_output(&establish_connection(), &mut std::io::stdout()).unwrap();
    rocket::build().mount(
        "/",
        routes![index, add, count, depth, sentences, pairs, orthos, delete, phrases, splat],
    )
}
