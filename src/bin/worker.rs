use std::collections::HashSet;

use polyvinyl_acetate::{get_relevant_vocabulary_reverse, worker_helper, Holder};

fn main() {
    let mut holder = Holder::default();

    let book = holder.insert_book("example".to_owned(), "a b. c d. a c. b d.".to_owned());
    holder.insert_todos("books", vec![book.id]);

    loop {
        let next_todo = holder.get_next_todo();

        if next_todo.is_none() {
            break;
        }

        worker_helper::handle_todo(next_todo.unwrap(), &mut holder)
    }

    get_relevant_vocabulary_reverse(&holder, HashSet::default());
}

// pub fn splat_orthos(dims: BTreeMap<usize, usize>) -> Result<String, anyhow::Error> {
//     let results = get_orthos_by_size(&establish_connection_safe()?, dims)?;

//     let phrases: Vec<_> = results
//         .iter()
//         .map(|o| o.all_full_length_phrases())
//         .collect();

//     let all_words: HashSet<Word> = phrases.iter().flatten().flatten().cloned().collect();
//     let mapping = get_relevant_vocabulary_reverse(&establish_connection_safe()?, all_words)?;

//     let res = phrases
//         .iter()
//         .map(|o| {
//             o.iter()
//                 .map(|s| {
//                     s.iter()
//                         .map(|w| mapping.get(w).expect("do not look up new words"))
//                         .cloned()
//                         .collect::<Vec<_>>()
//                         .join(" ")
//                 })
//                 .collect::<Vec<_>>()
//                 .join("\n")
//         })
//         .collect::<Vec<_>>()
//         .join("\n\n");

//     Ok(res)
// }