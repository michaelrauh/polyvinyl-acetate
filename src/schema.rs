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
        origin -> Int4,
        hop -> Array<Int4>,
        contents -> Array<Int4>,
        base -> Bool,
        info_hash -> Int8,
    }
}

table! {
    pairs (id) {
        id -> Int4,
        first_word -> Int4,
        second_word -> Int4,
        pair_hash -> Int8,
    }
}

table! {
    phrases (id) {
        id -> Int4,
        words -> Array<Int4>,
        phrase_head -> Int8,
        phrase_tail -> Int8,
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

table! {
    words (id) {
        id -> Int4,
        word -> Text,
        word_hash -> Int8,
    }
}

allow_tables_to_appear_in_same_query!(books, orthotopes, pairs, phrases, sentences, todos, words,);
