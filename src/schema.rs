table! {
    books (id) {
        id -> Int4,
        title -> Varchar,
        body -> Text,
    }
}

table! {
    orthotopes (id) {
        id -> Int4,
        information -> Bytea,
        origin -> Text,
        hop -> Array<Text>,
        contents -> Array<Text>,
        info_hash -> Int8,
    }
}

table! {
    pairs (id) {
        id -> Int4,
        first_word -> Text,
        second_word -> Text,
        pair_hash -> Int8,
    }
}

table! {
    phrases (id) {
        id -> Int4,
        words -> Array<Text>,
        words_hash -> Int8,
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

allow_tables_to_appear_in_same_query!(books, orthotopes, pairs, phrases, sentences, todos,);
