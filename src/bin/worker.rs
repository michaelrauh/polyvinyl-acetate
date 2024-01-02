use std::{collections::HashSet, fs};

use polyvinyl_acetate::{worker_helper, Holder};

// todo holder statistics
// todo flamegraph
// todo fix unit tests
// todo fix system tests
// todo harness
// todo disk backed KV?

fn main() {
    let mut holder = Holder::new();
    let f = fs::read_to_string("input.txt").unwrap();

    let book = holder.insert_book("example".to_owned(), f);
    holder.insert_todos("books", vec![book.id]);
    let mut i = 0;
    loop {
        i += 1;
        if i % 1000 == 0 {
            dbg!();
            dbg!(i);
            holder.get_stats();
        }
        let next_todo = holder.get_next_todo();

        if next_todo.is_none() {
            break;
        }

        worker_helper::handle_todo(next_todo.unwrap(), &mut holder)
    }

    {
        let holder = &holder;
        let words = HashSet::new();
        holder.get_vocabulary_slice_with_words(words)
    };
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
