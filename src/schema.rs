table! {
    books (id) {
        id -> Int4,
        title -> Varchar,
        body -> Text,
    }
}

table! {
    pairs (id) {
        id -> Int4,
        first_word -> Text,
        second_word -> Text,
        first_word_hash -> Int8,
        second_word_hash -> Int8,
    }
}

table! {
    sentences (id) {
        id -> Int4,
        sentence -> Text,
        sentence_hash -> Int8,
    }
}

table! {
    todos (id) {
        id -> Int4,
        domain -> Varchar,
        other -> Int4,
    }
}

allow_tables_to_appear_in_same_query!(books, pairs, sentences, todos,);
