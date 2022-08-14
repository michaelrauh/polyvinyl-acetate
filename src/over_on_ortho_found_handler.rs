use std::collections::HashSet;

use anyhow::Error;
use diesel::PgConnection;
use itertools::{iproduct, Itertools};
use maplit::{btreeset, hashmap};

use crate::{
    ortho::Ortho, phrase_ortho_handler::attempt_combine_over_with_phrases, vec_of_words_to_big_int,
    Word,
};

pub(crate) fn over_forward(
    conn: Option<&diesel::PgConnection>,
    old_orthotope: crate::ortho::Ortho,
    get_ortho_by_origin_batch: fn(
        Option<&PgConnection>,
        HashSet<Word>,
    ) -> Result<Vec<Ortho>, anyhow::Error>,
    project_forward_batch: fn(
        Option<&PgConnection>,
        HashSet<Word>,
    ) -> Result<HashSet<(Word, Word)>, Error>,
    get_phrases_with_matching_hashes: fn(
        Option<&PgConnection>,
        HashSet<i64>,
    ) -> Result<HashSet<i64>, anyhow::Error>,
    phrase_exists_db_filter_head: fn(
        Option<&PgConnection>,
        HashSet<i64>,
    ) -> Result<HashSet<i64>, anyhow::Error>,
) -> Result<Vec<crate::ortho::Ortho>, anyhow::Error> {
    let all_phrases = old_orthotope.origin_phrases();

    let lasts = all_phrases
        .iter()
        .map(|old_phrase| old_phrase.last().expect("orthos cannot have empty phrases"))
        .copied()
        .collect::<HashSet<_>>();
    let forwards = project_forward_batch(conn, lasts)?;

    let desired_phrases = Itertools::cartesian_product(all_phrases.iter(), forwards.iter()) // group before filter
        .filter(|(old_phrase, (f, _s))| {
            let last = old_phrase.last().expect("orthos cannot have empty phrases");
            last == f
        })
        .map(|(old_phrase, (_f, s))| {
            let current_phrase = old_phrase
                .clone()
                .iter()
                .chain(vec![*s].iter())
                .copied()
                .collect::<Vec<_>>();
            vec_of_words_to_big_int(current_phrase)
        })
        .collect::<HashSet<_>>();

    let actual_phrases = get_phrases_with_matching_hashes(conn, desired_phrases)?;

    let all_phrase_heads: HashSet<i64> = old_orthotope
        .all_full_length_phrases()
        .iter()
        .map(|p| vec_of_words_to_big_int(p.to_vec()))
        .collect();

    let speculative_potential_phrases = phrase_exists_db_filter_head(conn, all_phrase_heads)?;

    let mut ans: Vec<Ortho> = vec![];
    let all_second_words = all_phrases.iter().map(|p| p[1]).collect();
    let all_potential_orthos = get_ortho_by_origin_batch(conn, all_second_words)?;
    let mut phrase_to_ortho: std::collections::HashMap<
        &Vec<i32>,
        std::collections::BTreeSet<&Ortho>,
    > = hashmap! {};
    Itertools::cartesian_product(all_phrases.iter(), all_potential_orthos.iter()) // group before product
        .filter(|(phrase, ortho)| ortho.get_origin() == phrase[1])
        .for_each(|(phrase, ortho)| {
            phrase_to_ortho
                .entry(phrase)
                .and_modify(|s: &mut std::collections::BTreeSet<&Ortho>| {
                    s.insert(ortho);
                })
                .or_insert(btreeset! {ortho});
        });
    for old_phrase in all_phrases.clone() { // flatten this out
        let last = old_phrase.last().expect("orthos cannot have empty phrases");
        let nexts = forwards
            .iter()
            .filter(|(f, _s)| last == f)
            .copied()
            .map(|(_f, s)| s);
        for next in nexts {
            let current_phrase = old_phrase
                .clone()
                .iter()
                .chain(vec![next].iter())
                .copied()
                .collect::<Vec<_>>();
            if actual_phrases.contains(&vec_of_words_to_big_int(current_phrase.clone())) {
                for potential_ortho in phrase_to_ortho.get(&old_phrase).unwrap_or(&btreeset! {}) {
                    let phrase_tail = &current_phrase[1..];
                    if potential_ortho.origin_has_phrase(phrase_tail)
                        && potential_ortho.axis_length(current_phrase[2]) // fuse length check to existence
                            == current_phrase.len() - 2
                        && old_orthotope.get_dims() == potential_ortho.get_dims()
                    {
                        for found in attempt_combine_over_with_phrases(
                            &speculative_potential_phrases,
                            &old_orthotope,
                            potential_ortho,
                            current_phrase[1],
                            current_phrase[2],
                        ) {
                            ans.push(found);
                        }
                    }
                }
            }
        }
    }

    Ok(ans)
}

pub(crate) fn over_back(
    conn: Option<&diesel::PgConnection>,
    old_orthotope: crate::ortho::Ortho,
    get_ortho_by_origin_batch: fn(
        Option<&PgConnection>,
        HashSet<Word>,
    ) -> Result<Vec<Ortho>, anyhow::Error>,
    project_backward_batch: fn(
        Option<&PgConnection>,
        HashSet<Word>,
    ) -> Result<HashSet<(Word, Word)>, anyhow::Error>,
    get_phrases_with_matching_hashes: fn(
        Option<&PgConnection>,
        HashSet<i64>,
    ) -> Result<HashSet<i64>, anyhow::Error>,
    phrase_exists_db_filter_head: fn(
        Option<&PgConnection>,
        HashSet<i64>,
    ) -> Result<HashSet<i64>, anyhow::Error>,
) -> Result<Vec<crate::ortho::Ortho>, anyhow::Error> {
    let all_phrases = old_orthotope.origin_phrases();

    let firsts = all_phrases
        .iter()
        .map(|old_phrase| old_phrase[0])
        .collect::<HashSet<_>>();
    let backwards = project_backward_batch(conn, firsts)?;

    let desired_phrases = Itertools::cartesian_product(all_phrases.iter(), backwards.iter()) // group before product
        .filter(|(old_phrase, (_f, s))| {
            let first = old_phrase[0];
            first == *s
        })
        .map(|(old_phrase, (f, _s))| {
            let current_phrase = vec![*f]
                .iter()
                .chain(old_phrase.iter())
                .copied()
                .collect::<Vec<_>>();
            vec_of_words_to_big_int(current_phrase)
        })
        .collect::<HashSet<_>>();

    let actual_phrases = get_phrases_with_matching_hashes(conn, desired_phrases)?;
    let all_phrase_tails: HashSet<i64> = old_orthotope
        .all_full_length_phrases()
        .iter()
        .map(|p| vec_of_words_to_big_int(p.to_vec()))
        .collect();

    let speculative_potential_phrases = phrase_exists_db_filter_head(conn, all_phrase_tails)?;

    let all_first_words = backwards.iter().map(|(f, _s)| f).copied().collect();
    let all_potential_orthos = get_ortho_by_origin_batch(conn, all_first_words)?;
    let mut phrase_to_ortho: std::collections::HashMap<
        &Vec<i32>,
        std::collections::BTreeSet<&Ortho>,
    > = hashmap! {};
    iproduct!(
        all_phrases.iter(),
        all_potential_orthos.iter(),
        backwards.iter()
    )
    .filter(|(phrase, ortho, (f, s))| ortho.get_origin() == *f && phrase[0] == *s) // avoid this filter
    .for_each(|(phrase, ortho, _p)| {
        phrase_to_ortho
            .entry(phrase)
            .and_modify(|s: &mut std::collections::BTreeSet<&Ortho>| {
                s.insert(ortho);
            })
            .or_insert(btreeset! {ortho});
    });

    let mut ans: Vec<Ortho> = vec![];
    for old_phrase in all_phrases.clone() { // flatten this out
        let first = old_phrase[0];
        let prevs = backwards
            .iter()
            .filter(|(_f, s)| first == *s)
            .copied()
            .map(|(f, _s)| f)
            .collect::<Vec<_>>();
        for prev in prevs {
            let current_phrase = vec![prev]
                .iter()
                .chain(old_phrase.iter())
                .map(|x| x.to_owned())
                .collect::<Vec<_>>();
            if actual_phrases.contains(&vec_of_words_to_big_int(current_phrase.clone())) {
                for potential_ortho in phrase_to_ortho.get(&old_phrase).unwrap_or(&btreeset! {}) {
                    let phrase_head = &current_phrase[..current_phrase.len() - 1];
                    if potential_ortho.origin_has_phrase(phrase_head)
                        && potential_ortho.axis_length(current_phrase[1])
                            == current_phrase.len() - 2 // fuse length to phrase check 
                        && old_orthotope.get_dims() == potential_ortho.get_dims()
                    {
                        for found in attempt_combine_over_with_phrases(
                            &speculative_potential_phrases,
                            potential_ortho,
                            &old_orthotope,
                            current_phrase[1],
                            current_phrase[2],
                        ) {
                            ans.push(found);
                        }
                    }
                }
            }
        }
    }

    Ok(ans)
}

#[cfg(test)]
mod tests {
    use crate::{
        ortho::Ortho, over_on_ortho_found_handler::over_back,
        over_on_ortho_found_handler::over_forward, vec_of_words_to_big_int, Word,
    };
    use diesel::PgConnection;
    use maplit::{btreemap, hashset};
    use std::collections::HashSet;

    fn fake_forward_batch(
        _conn: Option<&PgConnection>,
        _from: HashSet<Word>,
    ) -> Result<HashSet<(Word, Word)>, anyhow::Error> {
        // a b  | b e
        // c d  | d f
        let pairs = hashset! { (2,4), (2, 5)};
        Ok(pairs)
    }

    fn fake_get_phrases_with_matching_hashes(
        _conn: Option<&PgConnection>,
        all_phrases: HashSet<i64>,
    ) -> Result<HashSet<i64>, anyhow::Error> {
        Ok(all_phrases)
    }

    fn fake_phrase_exists_db_filter_head(
        _conn: Option<&PgConnection>,
        _head: HashSet<i64>,
    ) -> Result<HashSet<i64>, anyhow::Error> {
        let res = hashset! {
            vec_of_words_to_big_int(vec![1, 2, 5]),
            vec_of_words_to_big_int(vec![3, 4, 6]),
        };
        Ok(res)
    }

    fn fake_backward_batch(
        _conn: Option<&PgConnection>,
        _from: HashSet<Word>,
    ) -> Result<HashSet<(Word, Word)>, anyhow::Error> {
        // a b  | b e
        // c d  | d f
        let pairs = hashset! { (1,2),};
        Ok(pairs)
    }

    fn fake_ortho_by_origin_batch(
        _conn: Option<&PgConnection>,
        _o: HashSet<Word>,
    ) -> Result<Vec<Ortho>, anyhow::Error> {
        let os = vec![Ortho::new(2, 5, 4, 6)];

        Ok(os)
    }

    fn fake_ortho_by_origin_two_batch(
        _conn: Option<&PgConnection>,
        _o: HashSet<Word>,
    ) -> Result<Vec<Ortho>, anyhow::Error> {
        let os = vec![Ortho::new(1, 2, 3, 4)];
        Ok(os)
    }

    #[test]
    fn it_creates_over_when_origin_points_to_origin_from_left() {
        // a b  | b e
        // c d  | d f

        let left_ortho = Ortho::new(1, 2, 3, 4);

        let right_ortho = Ortho::new(2, 5, 4, 6);

        let actual = over_forward(
            None,
            left_ortho.clone(),
            fake_ortho_by_origin_batch,
            fake_forward_batch,
            fake_get_phrases_with_matching_hashes,
            fake_phrase_exists_db_filter_head,
        )
        .unwrap();

        let expected = Ortho::zip_over(
            &left_ortho,
            &right_ortho,
            &btreemap! {
                5 => 2,
                4 => 3
            },
            5,
        );

        assert_eq!(actual, vec![expected]);
    }

    #[test]
    fn it_creates_over_when_origin_points_to_origin_from_right() {
        // a b  | b e
        // c d  | d f

        let left_ortho = Ortho::new(1, 2, 3, 4);

        let right_ortho = Ortho::new(2, 5, 4, 6);

        let actual = over_back(
            None,
            right_ortho.clone(),
            fake_ortho_by_origin_two_batch,
            fake_backward_batch,
            fake_get_phrases_with_matching_hashes,
            fake_phrase_exists_db_filter_head,
        )
        .unwrap();

        let expected = Ortho::zip_over(
            &left_ortho,
            &right_ortho,
            &btreemap! {
                5 => 2,
                4 => 3
            },
            5,
        );

        assert_eq!(actual, vec![expected]);
    }
}
