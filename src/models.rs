use serde::Deserialize;
use serde::Serialize;

use super::schema::books;
use super::schema::todos;


#[derive(Insertable)]
#[table_name = "books"]
pub struct NewBook {
    pub title: String,
    pub body: String,
}

#[derive(Queryable)]
pub struct Book {
    pub id: i32,
    pub title: String,
    pub body: String,
}

#[derive(Insertable)]
#[table_name = "todos"]
pub struct NewTodo {
    pub domain: String,
    pub other: i32,
}

#[derive(Queryable, Serialize, Deserialize, Debug)]
pub struct Todo {
    pub id: i32,
    pub domain: String,
    pub other: i32,
}
