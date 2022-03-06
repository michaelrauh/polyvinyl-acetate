extern crate polyvinyl_acetate;
extern crate diesel;

use self::polyvinyl_acetate::*;
use self::models::*;
use self::diesel::prelude::*;

fn main() {
    use polyvinyl_acetate::schema::posts::dsl::*;

    let connection = establish_connection();
    let results = posts.filter(published.eq(true))
        .limit(5)
        .load::<Post>(&connection)
        .expect("Error loading posts");

    println!("Displaying {} posts", results.len());
    for post in results {
        println!("{}", post.title);
        println!("----------\n");
        println!("{}", post.body);
    }
}
