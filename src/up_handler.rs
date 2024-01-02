use itertools::Itertools;
use maplit::hashset;
use std::collections::HashSet;

use crate::ortho::Ortho;

use crate::{ints_to_big_int, up_helper, Holder, Word};

pub fn up_by_origin(holder: &mut Holder, first_w: Word, second_w: Word) -> Vec<Ortho> {
    let left_orthos_by_origin: Vec<Ortho> = holder.get_base_orthos_with_origin(first_w);
    let right_orthos_by_origin: Vec<Ortho> = holder.get_base_orthos_with_origin(second_w);

    if left_orthos_by_origin.is_empty() || right_orthos_by_origin.is_empty() {
        return vec![];
    }

    let (all_firsts, all_seconds, all_pairs) = {
        let first_words = total_vocabulary(&left_orthos_by_origin);
        let second_words = total_vocabulary(&right_orthos_by_origin);
        let firsts: HashSet<(Word, Word, i64)> =
            holder.get_hashes_and_words_of_pairs_with_first_word(first_words);
        let seconds: HashSet<(Word, Word, i64)> =
            holder.get_hashes_and_words_of_pairs_with_second_word(second_words);

        let domain: HashSet<(Word, Word, i64)> = firsts.intersection(&seconds).cloned().collect();
        let mut firsts = hashset! {};
        let mut seconds = hashset! {};
        let mut hashes = hashset! {};
        domain.into_iter().for_each(|(f, s, h)| {
            firsts.insert(f);
            seconds.insert(s);
            hashes.insert(h);
        });
        (firsts, seconds, hashes)
    };

    let left_map =
        group_orthos_of_right_vocabulary_by_dimensionality(left_orthos_by_origin, all_firsts);
    let right_map =
        group_orthos_of_right_vocabulary_by_dimensionality(right_orthos_by_origin, all_seconds);

    attempt_up_for_pairs_of_matching_dimensionality(left_map, right_map, all_pairs)
}

pub fn up_by_hop(holder: &mut Holder, first_w: Word, second_w: Word) -> Vec<Ortho> {
    let hop_left_orthos: Vec<Ortho> = {
        let other_hop = vec![first_w];
        holder.get_base_orthos_with_hops_overlapping(other_hop)
    };
    let hop_right_orthos: Vec<Ortho> = {
        let other_hop = vec![second_w];
        holder.get_base_orthos_with_hops_overlapping(other_hop)
    };

    if hop_left_orthos.is_empty() || hop_right_orthos.is_empty() {
        return vec![];
    }

    find_corresponding_non_origin_checked_orthos_and_attempt_up(
        holder,
        hop_left_orthos,
        hop_right_orthos,
    )
}

pub fn up_by_contents(holder: &mut Holder, first_w: Word, second_w: Word) -> Vec<Ortho> {
    let contents_left_orthos: Vec<Ortho> = {
        let other_contents = vec![first_w];
        holder.get_base_orthos_with_contents_overlapping(other_contents)
    };
    let contents_right_orthos: Vec<Ortho> = {
        let other_contents = vec![second_w];
        holder.get_base_orthos_with_contents_overlapping(other_contents)
    };

    if contents_left_orthos.is_empty() || contents_right_orthos.is_empty() {
        return vec![];
    }

    find_corresponding_non_origin_checked_orthos_and_attempt_up(
        holder,
        contents_left_orthos,
        contents_right_orthos,
    )
}

fn attempt_up_for_pairs_of_matching_dimensionality(
    left_map: std::collections::HashMap<usize, Vec<Ortho>>,
    right_map: std::collections::HashMap<usize, Vec<Ortho>>,
    all_pairs: HashSet<i64>,
) -> Vec<Ortho> {
    let dimensionalities_left: HashSet<&usize> = HashSet::from_iter(left_map.keys());
    let dimensionalities_right: HashSet<&usize> = HashSet::from_iter(right_map.keys());
    let keys = dimensionalities_left.intersection(&dimensionalities_right);
    keys.flat_map(|dimensionality| {
        let suspect_left = left_map
            .get(dimensionality)
            .expect("key must exist as it is from this set");
        let suspect_right = right_map
            .get(dimensionality)
            .expect("key must exist as it is from this set");
        Itertools::cartesian_product(suspect_left.iter(), suspect_right.iter())
            .flat_map(|(lo, ro)| up_helper::attempt_up(&all_pairs, lo, ro))
    })
    .collect()
}

fn find_corresponding_non_origin_checked_orthos_and_attempt_up(
    holder: &mut Holder,
    hop_left_orthos: Vec<Ortho>,
    hop_right_orthos: Vec<Ortho>,
) -> Vec<Ortho> {
    let (all_firsts, all_seconds, all_pairs) = {
        let first_words = total_vocabulary(&hop_left_orthos);
        let second_words = total_vocabulary(&hop_right_orthos);
        let firsts: HashSet<(Word, Word, i64)> =
            holder.get_hashes_and_words_of_pairs_with_first_word(first_words);
        let seconds: HashSet<(Word, Word, i64)> =
            holder.get_hashes_and_words_of_pairs_with_second_word(second_words);

        let domain: HashSet<(Word, Word, i64)> = firsts.intersection(&seconds).cloned().collect();
        let mut firsts = hashset! {};
        let mut seconds = hashset! {};
        let mut hashes = hashset! {};
        domain.into_iter().for_each(|(f, s, h)| {
            firsts.insert(f);
            seconds.insert(s);
            hashes.insert(h);
        });
        (firsts, seconds, hashes)
    };
    let left_map = group_orthos_of_right_vocabulary_by_dimensionality(hop_left_orthos, all_firsts);
    let right_map =
        group_orthos_of_right_vocabulary_by_dimensionality(hop_right_orthos, all_seconds);
    let res = attempt_up_for_pairs_of_matching_dimensionality_if_origin_mapping_exists(
        left_map, right_map, all_pairs,
    );
    res
}

fn attempt_up_for_pairs_of_matching_dimensionality_if_origin_mapping_exists(
    left_map: std::collections::HashMap<usize, Vec<Ortho>>,
    right_map: std::collections::HashMap<usize, Vec<Ortho>>,
    all_pairs: HashSet<i64>,
) -> Vec<Ortho> {
    let dimensionalities_left: HashSet<&usize> = HashSet::from_iter(left_map.keys());
    let dimensionalities_right: HashSet<&usize> = HashSet::from_iter(right_map.keys());

    dimensionalities_left
        .intersection(&dimensionalities_right)
        .flat_map(|dimensionality| {
            Itertools::cartesian_product(
                left_map
                    .get(dimensionality)
                    .expect("key must exist as it is from this set")
                    .iter(),
                right_map
                    .get(dimensionality)
                    .expect("key must exist as it is from this set")
                    .iter(),
            )
            .filter(|(lo, ro)| {
                all_pairs.contains(&ints_to_big_int(lo.get_origin(), ro.get_origin()))
            })
            .flat_map(|(lo, ro)| up_helper::attempt_up(&all_pairs, lo, ro))
        })
        .collect()
}

fn group_orthos_of_right_vocabulary_by_dimensionality(
    orthos: Vec<Ortho>,
    vocabulary: HashSet<i32>,
) -> std::collections::HashMap<usize, Vec<Ortho>> {
    Itertools::into_group_map_by(
        orthos
            .into_iter()
            .filter(|o| o.get_vocabulary().all(|w| vocabulary.contains(&w))),
        |o| o.get_dimensionality(),
    )
}

fn total_vocabulary(orthos: &[Ortho]) -> HashSet<i32> {
    orthos.iter().flat_map(|lo| lo.get_vocabulary()).collect()
}

#[cfg(test)]
mod tests {

    use crate::ortho::Ortho;
    use crate::up_handler::{up_by_contents, up_by_hop, up_by_origin};
    use crate::Holder;
    use maplit::btreemap;

    #[test]
    fn it_creates_up_on_pair_add_when_origin_points_to_origin() {
        let mut holder: Holder = Holder::new();
        let actual = up_by_origin(&mut holder, 1, 5);

        let expected = Ortho::zip_up(
            &Ortho::new(1, 2, 3, 4),
            &Ortho::new(5, 6, 7, 8),
            &btreemap! {
                5 => 1,
                6 => 2,
                7 => 3
            },
        );

        assert_eq!(actual, vec![expected]);
    }

    #[test]
    fn it_does_not_create_up_when_a_forward_is_missing() {
        let mut holder: Holder = Holder::new();
        let actual = up_by_origin(&mut holder, 1, 5);

        assert_eq!(actual, vec![]);
    }

    #[test]
    fn it_does_not_produce_up_if_that_would_create_a_diagonal_conflict() {
        let mut holder: Holder = Holder::new();
        let actual = up_by_origin(&mut holder, 1, 5);

        assert_eq!(actual, vec![]);
    }

    #[test]
    fn it_does_not_produce_up_for_non_base_dims_even_if_eligible() {
        let mut holder: Holder = Holder::new();
        let actual = up_by_origin(&mut holder, 1, 7);

        assert_eq!(actual, vec![]);
    }

    #[test]
    fn it_only_attempts_to_combine_same_dim_orthos() {
        let mut holder: Holder = Holder::new();
        let actual = up_by_origin(&mut holder, 1, 5);

        assert_eq!(actual, vec![]);
    }

    #[test]
    fn it_attempts_to_combine_by_hop() {
        // same combine as before, but b -> f is the pair so it must index into hops
        let mut holder: Holder = Holder::new();
        let actual = up_by_hop(
            &mut holder,
            2, // a b c d + e f g h
            6,
        );

        let expected = Ortho::zip_up(
            &Ortho::new(1, 2, 3, 4),
            &Ortho::new(5, 6, 7, 8),
            &btreemap! {
                5 => 1,
                6 => 2,
                7 => 3
            },
        );

        assert_eq!(actual, vec![expected]);
    }

    #[test]
    fn it_attempts_to_combine_by_contents() {
        // same combine as before, but d -> h is the pair so it must index into contents
        let mut holder: Holder = Holder::new();
        let actual = up_by_contents(
            &mut holder,
            4, // a b c d + e f g h
            8,
        );

        let expected = Ortho::zip_up(
            &Ortho::new(1, 2, 3, 4),
            &Ortho::new(5, 6, 7, 8),
            &btreemap! {
                5 => 1,
                6 => 2,
                7 => 3
            },
        );

        assert_eq!(actual, vec![expected]);
    }
}
