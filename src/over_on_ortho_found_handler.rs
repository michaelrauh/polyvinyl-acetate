use std::collections::HashSet;

use itertools::Itertools;
use maplit::{btreeset, hashmap};

use crate::{
    ortho::Ortho, phrase_ortho_handler::attempt_combine_over_with_phrases, vec_of_words_to_big_int,
    Holder, Word,
};

pub(crate) fn over_forward(
    holder: &mut Holder,
    old_orthotope: crate::ortho::Ortho,
) -> Vec<crate::ortho::Ortho> {
    let all_phrases = old_orthotope.origin_phrases();
    let all_second_words = all_phrases.iter().map(|p| p[1]).collect();
    let all_potential_orthos = holder.get_ortho_with_origin_in(all_second_words);

    if all_potential_orthos.is_empty() {
        return vec![];
    }

    let lasts = all_phrases
        .iter()
        .map(|old_phrase| old_phrase.last().expect("orthos cannot have empty phrases"))
        .copied()
        .collect::<HashSet<_>>();
    let forwards = holder.get_words_of_pairs_with_first_word_in(lasts);

    let last_to_phrase = Itertools::into_group_map_by(all_phrases.iter(), |old_phrase| {
        old_phrase.last().expect("orthos cannot have empty phrases")
    });
    let first_to_pair = Itertools::into_group_map_by(forwards.iter(), |(f, _s)| f);

    let phrase_keys: HashSet<Word> = HashSet::from_iter(last_to_phrase.keys().copied().copied());
    let pair_keys: HashSet<Word> = HashSet::from_iter(first_to_pair.keys().copied().copied());

    let desired_phrases = phrase_keys
        .intersection(&pair_keys)
        .flat_map(|overlap_word| {
            let old_phrases = last_to_phrase
                .get(overlap_word)
                .expect("dont ask for a nonexistent key in an intersect");
            let second_words = first_to_pair
                .get(overlap_word)
                .expect("dont ask for a nonexistent key in an intersect")
                .iter()
                .map(|(_f, s)| s);

            Itertools::cartesian_product(old_phrases.iter(), second_words)
                .map(|(old_phrase, second_word)| {
                    let current_phrase = old_phrase
                        .iter()
                        .chain(vec![*second_word].iter())
                        .copied()
                        .collect::<Vec<_>>();
                    vec_of_words_to_big_int(current_phrase)
                })
                .collect_vec()
        })
        .collect::<HashSet<_>>();

    let actual_phrases: Vec<_> = holder
        .get_phrases_matching(desired_phrases)
        .iter()
        .cloned()
        .collect();

    let all_phrase_heads: HashSet<i64> = old_orthotope
        .all_full_length_phrases()
        .iter()
        .map(|p| vec_of_words_to_big_int(p.to_vec()))
        .collect();

    let speculative_potential_phrases =
        holder.get_phrase_hash_with_phrase_head_matching(all_phrase_heads);

    let mut ans: Vec<Ortho> = vec![];

    let mut phrase_to_ortho: std::collections::HashMap<
        &Vec<i32>,
        std::collections::BTreeSet<&Ortho>,
    > = hashmap! {};

    let second_to_phrase = Itertools::into_group_map_by(all_phrases.iter(), |phrase| phrase[1]);
    let origin_to_ortho =
        Itertools::into_group_map_by(all_potential_orthos.iter(), |ortho| ortho.get_origin());

    let second_to_phrase_keys: HashSet<Word> = HashSet::from_iter(second_to_phrase.keys().copied());
    let origin_to_ortho_keys: HashSet<Word> = HashSet::from_iter(origin_to_ortho.keys().copied());

    second_to_phrase_keys
        .intersection(&origin_to_ortho_keys)
        .for_each(|overlap_word| {
            let phrases = second_to_phrase
                .get(overlap_word)
                .expect("this is on a key intersect");
            let orthos = origin_to_ortho
                .get(overlap_word)
                .expect("this is on a key intersect");

            Itertools::cartesian_product(phrases.iter(), orthos.iter()).for_each(
                |(phrase, ortho)| {
                    phrase_to_ortho
                        .entry(phrase)
                        .and_modify(|s: &mut std::collections::BTreeSet<&Ortho>| {
                            s.insert(ortho);
                        })
                        .or_insert(btreeset! {*ortho});
                },
            );
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
                .iter()
                .chain(vec![next].iter())
                .copied()
                .collect::<Vec<_>>();
            if actual_phrases.contains(&vec_of_words_to_big_int(current_phrase.clone())) {
                for potential_ortho in phrase_to_ortho.get(&old_phrase).unwrap_or(&btreeset! {}) {
                    let phrase_tail = &current_phrase[1..];
                    if potential_ortho.origin_has_full_length_phrase(phrase_tail)
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

    ans
}

pub(crate) fn over_back(
    holder: &mut Holder,
    old_orthotope: crate::ortho::Ortho,
) -> Vec<crate::ortho::Ortho> {
    let all_phrases = old_orthotope.origin_phrases();

    let firsts = all_phrases
        .iter()
        .map(|old_phrase| old_phrase[0])
        .collect::<HashSet<_>>();
    let backwards = holder.get_words_of_pairs_with_second_word_in(firsts);

    let all_first_words = backwards.iter().map(|(f, _s)| f).copied().collect();
    let all_potential_orthos = holder.get_ortho_with_origin_in(all_first_words);

    if all_potential_orthos.is_empty() {
        return vec![];
    }

    let first_to_phrase = Itertools::into_group_map_by(all_phrases.iter(), |phrase| phrase[0]);
    let second_to_backwards = Itertools::into_group_map_by(backwards.iter(), |(_f, s)| s);

    let first_to_phrase_keys: HashSet<Word> = HashSet::from_iter(first_to_phrase.keys().copied());
    let second_to_backwards_keys: HashSet<Word> =
        HashSet::from_iter(second_to_backwards.keys().copied().copied());

    let desired_phrases = first_to_phrase_keys
        .intersection(&second_to_backwards_keys)
        .flat_map(|overlap_word| {
            let phrases = first_to_phrase
                .get(overlap_word)
                .expect("this is on a key intersect");
            let backwards = second_to_backwards
                .get(overlap_word)
                .expect("this is on a key intersect");

            Itertools::cartesian_product(phrases.iter(), backwards.iter())
                .map(|(phrase, (f, _s))| {
                    let current_phrase = vec![*f]
                        .iter()
                        .chain(phrase.iter())
                        .copied()
                        .collect::<Vec<_>>();
                    vec_of_words_to_big_int(current_phrase)
                })
                .collect_vec()
        })
        .collect::<HashSet<_>>();

    let actual_phrases: Vec<_> = holder
        .get_phrases_matching(desired_phrases)
        .iter()
        .cloned()
        .collect();
    let all_phrase_tails: HashSet<i64> = old_orthotope
        .all_full_length_phrases()
        .iter()
        .map(|p| vec_of_words_to_big_int(p.to_vec()))
        .collect();

    let speculative_potential_phrases =
        holder.get_phrase_hash_with_phrase_tail_matching(all_phrase_tails);

    let mut phrase_to_ortho: std::collections::HashMap<
        &Vec<i32>,
        std::collections::BTreeSet<&Ortho>,
    > = hashmap! {};

    let all_firsts = backwards
        .iter()
        .map(|(f, _s)| *f)
        .collect::<HashSet<Word>>();
    let all_seconds = backwards
        .iter()
        .map(|(_f, s)| *s)
        .collect::<HashSet<Word>>();
    let origin_to_ortho =
        Itertools::into_group_map_by(all_potential_orthos.iter(), |ortho| ortho.get_origin());
    let phrase_head_to_phrase =
        Itertools::into_group_map_by(all_phrases.iter(), |phrase| phrase[0]);

    let origin_to_ortho_keys: HashSet<Word> = HashSet::from_iter(origin_to_ortho.keys().copied());
    let phrase_head_to_phrase_keys: HashSet<Word> =
        HashSet::from_iter(phrase_head_to_phrase.keys().copied());

    let relevant_origin_to_ortho_keys = all_firsts.intersection(&origin_to_ortho_keys);
    let relevant_phrase_head_to_phrase_keys = all_seconds.intersection(&phrase_head_to_phrase_keys);

    Itertools::cartesian_product(
        relevant_origin_to_ortho_keys,
        relevant_phrase_head_to_phrase_keys,
    )
    .for_each(|(origin, phrase_head)| {
        let phrases = phrase_head_to_phrase
            .get(phrase_head)
            .expect("intersection on key");
        let orthos = origin_to_ortho.get(origin).expect("intersection on key");

        Itertools::cartesian_product(phrases.iter(), orthos.iter()).for_each(|(phrase, ortho)| {
            phrase_to_ortho
                .entry(phrase)
                .and_modify(|s: &mut std::collections::BTreeSet<&Ortho>| {
                    s.insert(ortho);
                })
                .or_insert(btreeset! {*ortho});
        })
    });

    let mut ans: Vec<Ortho> = vec![];
    for old_phrase in all_phrases.clone() {
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
                    if potential_ortho.origin_has_full_length_phrase(phrase_head)
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

    ans
}

#[cfg(test)]
mod tests {
    use crate::{
        ortho::Ortho, over_on_ortho_found_handler::over_back,
        over_on_ortho_found_handler::over_forward, Holder,
    };

    use maplit::btreemap;

    #[test]
    fn it_creates_over_when_origin_points_to_origin_from_left() {
        // a b  | b e
        // c d  | d f

        let left_ortho = Ortho::new(1, 2, 3, 4);

        let right_ortho = Ortho::new(2, 5, 4, 6);
        let mut holder = Holder::new();
        let actual = over_forward(&mut holder, left_ortho.clone());

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
        let mut holder = Holder::new();
        let actual = over_back(&mut holder, right_ortho.clone());

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
