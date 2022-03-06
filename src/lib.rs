pub mod schema;
pub mod models;

#[macro_use]
extern crate diesel;
extern crate dotenv;

use diesel::prelude::*;
use diesel::pg::PgConnection;
use dotenv::dotenv;
use std::env;
use crate::schema::posts::{dsl::posts, published};

pub fn establish_connection() -> PgConnection {
    dotenv().ok();

    let database_url = env::var("DATABASE_URL")
        .expect("DATABASE_URL must be set");
    PgConnection::establish(&database_url)
        .expect(&format!("Error connecting to {}", database_url))
}

use self::models::{Post, NewPost};

pub fn create_post<'a>(conn: &PgConnection, title: &'a str, body: &'a str) -> Post {
    use schema::posts;

    let new_post = NewPost {
        title: title,
        body: body,
    };

    diesel::insert_into(posts::table)
        .values(&new_post)
        .get_result(conn)
        .expect("Error saving new post")
}

pub fn show_posts() -> String {
    let connection = establish_connection();
    let results = posts.filter(published.eq(true))
        .limit(5)
        .load::<Post>(&connection)
        .expect("Error loading posts");
    println!("Displaying {} posts", results.len());
    for post in results.clone() {
        println!("{}", post.title);
        println!("----------\n");
        println!("{}", post.body);
    }
    let mut res = vec![];
    for post in results {
        res.push(post.title);
    }
    res.join("\n")
}