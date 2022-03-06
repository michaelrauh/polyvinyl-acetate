use polyvinyl_acetate::show_posts;

#[macro_use] extern crate rocket;

#[get("/")]
fn index() -> String {
    show_posts()
}

#[launch]
fn rocket() -> _ {
    rocket::build().mount("/", routes![index])
}