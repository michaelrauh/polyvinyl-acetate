use itertools::Itertools;
use std::collections::HashSet;

use crate::ortho::Ortho;

use crate::{ints_to_big_int, up_helper, Holder, Word};

pub fn up_by_origin(
    holder: &mut Holder,
    first_w: Word,
    second_w: Word,
    get_base_ortho_by_origin: fn(&mut Holder, Word) -> Vec<Ortho>,
    get_hashes_and_words_of_pairs_with_words_in: fn(
        &Holder,
        HashSet<i32>,
        HashSet<i32>,
    )
        -> (HashSet<Word>, HashSet<Word>, HashSet<i64>),
) -> Vec<Ortho> {
    let left_orthos_by_origin: Vec<Ortho> = get_base_ortho_by_origin(holder, first_w);
    let right_orthos_by_origin: Vec<Ortho> = get_base_ortho_by_origin(holder, second_w);

    if left_orthos_by_origin.is_empty() || right_orthos_by_origin.is_empty() {
        return vec![];
    }

    let (all_firsts, all_seconds, all_pairs) = get_hashes_and_words_of_pairs_with_words_in(
        holder,
        total_vocabulary(&left_orthos_by_origin),
        total_vocabulary(&right_orthos_by_origin),
    );

    let left_map =
        group_orthos_of_right_vocabulary_by_dimensionality(left_orthos_by_origin, all_firsts);
    let right_map =
        group_orthos_of_right_vocabulary_by_dimensionality(right_orthos_by_origin, all_seconds);

    attempt_up_for_pairs_of_matching_dimensionality(left_map, right_map, all_pairs)
}

pub fn up_by_hop(
    holder: &mut Holder,
    first_w: Word,
    second_w: Word,
    get_base_ortho_by_hop: fn(&Holder, Vec<Word>) -> Vec<Ortho>,
    get_hashes_and_words_of_pairs_with_words_in: fn(
        &Holder,
        HashSet<i32>,
        HashSet<i32>,
    )
        -> (HashSet<Word>, HashSet<Word>, HashSet<i64>),
) -> Vec<Ortho> {
    let hop_left_orthos: Vec<Ortho> = get_base_ortho_by_hop(holder, vec![first_w]);
    let hop_right_orthos: Vec<Ortho> = get_base_ortho_by_hop(holder, vec![second_w]);

    if hop_left_orthos.is_empty() || hop_right_orthos.is_empty() {
        return vec![];
    }

    find_corresponding_non_origin_checked_orthos_and_attempt_up(
        get_hashes_and_words_of_pairs_with_words_in,
        holder,
        hop_left_orthos,
        hop_right_orthos,
    )
}

pub fn up_by_contents(
    holder: &mut Holder,
    first_w: Word,
    second_w: Word,
    get_base_ortho_by_contents: fn(&Holder, Vec<Word>) -> Vec<Ortho>,
    get_hashes_and_words_of_pairs_with_words_in: fn(
        &Holder,
        HashSet<i32>,
        HashSet<i32>,
    )
        -> (HashSet<Word>, HashSet<Word>, HashSet<i64>),
) -> Vec<Ortho> {
    let contents_left_orthos: Vec<Ortho> = get_base_ortho_by_contents(holder, vec![first_w]);
    let contents_right_orthos: Vec<Ortho> = get_base_ortho_by_contents(holder, vec![second_w]);

    if contents_left_orthos.is_empty() || contents_right_orthos.is_empty() {
        return vec![];
    }

    find_corresponding_non_origin_checked_orthos_and_attempt_up(
        get_hashes_and_words_of_pairs_with_words_in,
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
    get_hashes_and_words_of_pairs_with_words_in: fn(
        &Holder,
        HashSet<i32>,
        HashSet<i32>,
    )
        -> (HashSet<Word>, HashSet<Word>, HashSet<i64>),
    holder: &mut Holder,
    hop_left_orthos: Vec<Ortho>,
    hop_right_orthos: Vec<Ortho>,
) -> Vec<Ortho> {
    let (all_firsts, all_seconds, all_pairs) = get_hashes_and_words_of_pairs_with_words_in(
        holder,
        total_vocabulary(&hop_left_orthos),
        total_vocabulary(&hop_right_orthos),
    );
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
    use std::collections::HashSet;

    use crate::ortho::Ortho;
    use crate::up_handler::{up_by_contents, up_by_hop, up_by_origin};
    use crate::{ints_to_big_int, Holder, Word};
    use maplit::{btreemap, hashset};

    fn fake_ortho_by_origin_two(_holder: &mut Holder, o: Word) -> Vec<Ortho> {
        let mut pairs = btreemap! { 1 => vec![Ortho::new(
            1,
            2,
            3,
            3,
        )], 5 => vec![Ortho::new(
            5,
            6,
            3,
            8,
        )]};
        pairs.entry(o).or_default().to_owned()
    }

    fn fake_ortho_by_origin_four(_holder: &mut Holder, o: Word) -> Vec<Ortho> {
        let single = Ortho::new(1, 2, 3, 4);
        let l_one = Ortho::new(5, 6, 7, 8);

        let r_one = Ortho::new(9, 10, 11, 12);

        let combined = Ortho::zip_up(&l_one, &r_one, &btreemap! { 10 => 6, 11 => 7 });

        let mut pairs = btreemap! { 1 => vec![single], 5 => vec![combined]};

        pairs.entry(o).or_default().to_owned()
    }

    fn fake_ortho_by_origin_three(_holder: &mut Holder, o: Word) -> Vec<Ortho> {
        let l_one = Ortho::new(1, 2, 4, 5);
        let l_two = Ortho::new(2, 3, 5, 6);
        let left_ortho = Ortho::zip_over(&l_one, &l_two, &btreemap! { 3 => 2, 5 => 4 }, 3);
        let r_one = Ortho::new(7, 8, 10, 11);
        let r_two = Ortho::new(8, 9, 11, 12);
        let r = Ortho::zip_over(&r_one, &r_two, &btreemap! { 9 => 8, 12 => 10 }, 9);
        let mut pairs = btreemap! { 1 => vec![left_ortho], 7 => vec![r]};

        pairs.entry(o).or_default().to_owned()
    }

    fn fake_ortho_by_origin(_holder: &mut Holder, o: Word) -> Vec<Ortho> {
        let mut pairs = btreemap! { 1 => vec![Ortho::new(
            1,
            2,
            3,
            4,
        )], 5 => vec![Ortho::new(
            5,
            6,
            7,
            8,
        )]};
        pairs.entry(o).or_default().to_owned()
    }

    fn fake_ortho_by_hop(_holder: &Holder, o: Vec<Word>) -> Vec<Ortho> {
        let mut ans = vec![];

        if o.contains(&2) {
            ans.push(Ortho::new(1, 2, 3, 4))
        }

        if o.contains(&6) {
            ans.push(Ortho::new(5, 6, 7, 8))
        }

        ans
    }

    fn fake_ortho_by_contents(_holder: &Holder, o: Vec<Word>) -> Vec<Ortho> {
        let mut ans = vec![];

        if o.contains(&4) {
            ans.push(Ortho::new(1, 2, 3, 4))
        }

        if o.contains(&8) {
            ans.push(Ortho::new(5, 6, 7, 8))
        }

        ans
    }

    fn fake_get_hashes_and_words_of_pairs_with_words_in(
        _holder: &Holder,
        _first_words: HashSet<Word>,
        _second_words: HashSet<Word>,
    ) -> (HashSet<Word>, HashSet<Word>, HashSet<i64>) {
        let pairs = vec![
            (1, 2),
            (3, 4),
            (1, 3),
            (2, 4),
            (5, 6),
            (7, 8),
            (5, 7),
            (6, 8),
            (1, 5),
            (2, 6),
            (3, 7),
            (4, 8),
        ];
        let res = pairs.iter().map(|(l, r)| ints_to_big_int(*l, *r)).collect();
        (
            hashset! {1, 3, 2, 5, 7, 6, 4},
            hashset! {2, 4, 3, 6, 8, 7, 5},
            res,
        )
    }

    fn fake_get_hashes_and_words_of_pairs_with_words_in_two(
        _holder: &Holder,
        _first_words: HashSet<Word>,
        _second_words: HashSet<Word>,
    ) -> (HashSet<Word>, HashSet<Word>, HashSet<i64>) {
        let pairs = vec![
            (1, 2),
            (3, 4),
            (1, 3),
            (2, 4),
            (5, 6),
            (7, 8),
            (5, 7),
            (6, 8),
            (1, 5),
            (2, 6),
            (3, 7),
        ];
        let res = pairs.iter().map(|(l, r)| ints_to_big_int(*l, *r)).collect();
        (
            hashset! {1, 3, 2, 5, 7, 6},
            hashset! {2, 4, 3, 6, 8, 7, 5},
            res,
        )
    }

    #[test]
    fn it_creates_up_on_pair_add_when_origin_points_to_origin() {
        let mut holder: Holder = Holder::new();
        let actual = up_by_origin(
            &mut holder,
            1,
            5,
            fake_ortho_by_origin,
            fake_get_hashes_and_words_of_pairs_with_words_in,
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
    fn it_does_not_create_up_when_a_forward_is_missing() {
        let mut holder: Holder = Holder::new();
        let actual = up_by_origin(
            &mut holder,
            1,
            5,
            fake_ortho_by_origin,
            fake_get_hashes_and_words_of_pairs_with_words_in_two,
        );

        assert_eq!(actual, vec![]);
    }

    #[test]
    fn it_does_not_produce_up_if_that_would_create_a_diagonal_conflict() {
        let mut holder: Holder = Holder::new();
        let actual = up_by_origin(
            &mut holder,
            1,
            5,
            fake_ortho_by_origin_two,
            fake_get_hashes_and_words_of_pairs_with_words_in,
        );

        assert_eq!(actual, vec![]);
    }

    #[test]
    fn it_does_not_produce_up_for_non_base_dims_even_if_eligible() {
        let mut holder: Holder = Holder::new();
        let actual = up_by_origin(
            &mut holder,
            1,
            7,
            fake_ortho_by_origin_three,
            fake_get_hashes_and_words_of_pairs_with_words_in,
        );

        assert_eq!(actual, vec![]);
    }

    #[test]
    fn it_only_attempts_to_combine_same_dim_orthos() {
        let mut holder: Holder = Holder::new();
        let actual = up_by_origin(
            &mut holder,
            1,
            5,
            fake_ortho_by_origin_four,
            fake_get_hashes_and_words_of_pairs_with_words_in,
        );

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
            fake_ortho_by_hop,
            fake_get_hashes_and_words_of_pairs_with_words_in,
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
            fake_ortho_by_contents,
            fake_get_hashes_and_words_of_pairs_with_words_in,
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
