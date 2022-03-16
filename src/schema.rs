table! {
    books (id) {
        id -> Int4,
        title -> Varchar,
        body -> Text,
    }
}

table! {
    todos (id) {
        id -> Int4,
        domain -> Varchar,
        other -> Int4,
    }
}

allow_tables_to_appear_in_same_query!(
    books,
    todos,
);
