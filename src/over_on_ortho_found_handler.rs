use std::collections::HashSet;

use anyhow::Error;
use diesel::PgConnection;
use itertools::Itertools;
use maplit::{hashmap, btreeset};

use crate::{
    ortho::Ortho, phrase_ortho_handler::attempt_combine_over, vec_of_words_to_big_int,
    FailableWordToOrthoVec, Word,
};

pub(crate) fn over_forward(
    conn: Option<&diesel::PgConnection>,
    old_orthotope: crate::ortho::Ortho,
    get_ortho_by_origin_batch: fn (
        Option<&PgConnection>,
        HashSet<Word>,
    ) -> Result<Vec<Ortho>, anyhow::Error> ,
    phrase_exists: fn(Option<&PgConnection>, Vec<Word>) -> Result<bool, anyhow::Error>,
    project_forward_batch: fn(
        Option<&PgConnection>,
        HashSet<Word>,
    ) -> Result<HashSet<(Word, Word)>, Error>,
    get_phrases_with_matching_hashes: fn(
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

    let desired_phrases = Itertools::cartesian_product(all_phrases.iter(), forwards.iter())
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

    let mut ans: Vec<Ortho> = vec![];
    let all_second_words = all_phrases.iter().map(|p| p[1]).collect();
    let all_potential_orthos = get_ortho_by_origin_batch(conn, all_second_words)?;
    let mut phrase_to_ortho: std::collections::HashMap<&Vec<i32>, std::collections::BTreeSet<&Ortho>> = hashmap!{};
    Itertools::cartesian_product(all_phrases.iter(), all_potential_orthos.iter()).filter(|(phrase, ortho)| {
        ortho.get_origin() == phrase[1]
    }).for_each(|(phrase, ortho)| {
        phrase_to_ortho.entry(phrase).and_modify(|s: &mut std::collections::BTreeSet<&Ortho>| {s.insert(ortho);}).or_insert(btreeset!{ortho});
    });
    for old_phrase in all_phrases.clone() {
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
                for potential_ortho in phrase_to_ortho.get(&old_phrase).unwrap_or(&btreeset!{}) {
                    let phrase_tail = &current_phrase[1..];
                    if potential_ortho.origin_has_phrase(phrase_tail)
                        && potential_ortho.axis_length(current_phrase[2]) == current_phrase.len() - 2
                        && old_orthotope.get_dims() == potential_ortho.get_dims()
                    {
                        for found in attempt_combine_over(
                            conn,
                            phrase_exists,
                            &old_orthotope,
                            &potential_ortho,
                            current_phrase[1],
                            current_phrase[2],
                        )? {
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
    get_ortho_by_origin: FailableWordToOrthoVec,
    phrase_exists: fn(Option<&PgConnection>, Vec<Word>) -> Result<bool, anyhow::Error>,
    project_backward_batch: fn(
        Option<&PgConnection>,
        HashSet<Word>,
    ) -> Result<HashSet<(Word, Word)>, anyhow::Error>,
    get_phrases_with_matching_hashes: fn(
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

    let desired_phrases = Itertools::cartesian_product(all_phrases.iter(), backwards.iter())
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

    let mut ans: Vec<Ortho> = vec![];
    for old_phrase in all_phrases {
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
                let phrase = current_phrase.to_vec();
                for potential_ortho in get_ortho_by_origin(conn, phrase[0])? {
                    let phrase_head = &phrase[..phrase.len() - 1];
                    if potential_ortho.origin_has_phrase(phrase_head)
                        && potential_ortho.axis_length(phrase[1]) == phrase.len() - 2
                        && old_orthotope.get_dims() == potential_ortho.get_dims()
                    {
                        for found in attempt_combine_over(
                            conn,
                            phrase_exists,
                            &potential_ortho,
                            &old_orthotope,
                            phrase[1],
                            phrase[2],
                        )? {
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
        over_on_ortho_found_handler::over_forward, Word,
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
        let os = vec![Ortho::new(
            2,
            5,
            4,
            6,
        )];

        Ok(os)
    }

    fn fake_ortho_by_origin_two(
        _conn: Option<&PgConnection>,
        o: Word,
    ) -> Result<Vec<Ortho>, anyhow::Error> {
        let mut pairs = btreemap! { 1 => vec![Ortho::new(
            1,
            2,
            3,
            4,
        )]};
        Ok(pairs.entry(o).or_default().to_owned())
    }

    fn fake_phrase_exists(
        _conn: Option<&PgConnection>,
        phrase: Vec<Word>,
    ) -> Result<bool, anyhow::Error> {
        let ps = hashset! {
            vec![1, 2, 5],
            vec![3, 4, 6]
        };
        Ok(ps.contains(&phrase))
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
            fake_phrase_exists,
            fake_forward_batch,
            fake_get_phrases_with_matching_hashes,
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
            fake_ortho_by_origin_two,
            fake_phrase_exists,
            fake_backward_batch,
            fake_get_phrases_with_matching_hashes,
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
