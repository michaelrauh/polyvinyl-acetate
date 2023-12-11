use polyvinyl_acetate::{worker_helper, Holder, models::Todo};

fn main() {
    let mut holder = Holder::new();

    let book = holder.insert_book("example".to_owned(), "a b. c d. a c. b d.".to_owned());
    
    worker_helper::handle_todo(Todo { id: 1, domain: "books".to_owned(), other: book.id }, &mut holder)
}