use crate::{ints_to_big_int, ortho::Ortho, Word};

use itertools::{zip, Itertools};

use std::collections::{BTreeMap, HashSet};

pub fn attempt_up(all_pairs: &HashSet<i64>, lo: &Ortho, ro: &Ortho) -> Vec<Ortho> {
    let _ = tracing::info_span!("try this one").entered();
    let lo_hop = lo.get_hop();
    let lo_hop_len = lo_hop.len();
    let left_hand_coordinate_configurations =
        Itertools::permutations(lo_hop.into_iter(), lo_hop_len);
    let fixed_right_hand: Vec<Word> = ro.get_hop().into_iter().collect();
    left_hand_coordinate_configurations
        .filter(|left_mapping| mapping_works(left_mapping, &fixed_right_hand, all_pairs))
        .map(|left_mapping| make_mapping(&left_mapping, &fixed_right_hand))
        .filter(|mapping| {
            diagonals_do_not_conflict(lo, ro) && mapping_is_complete(all_pairs, mapping, lo, ro)
        })
        .map(|mapping| Ortho::zip_up(lo, ro, &mapping))
        .collect()
}

fn diagonals_do_not_conflict(lo: &Ortho, ro: &Ortho) -> bool {
    for dist in 0..lo.get_dimensionality() {
        let lns = lo.get_names_at_distance(dist + 1);
        let rns = ro.get_names_at_distance(dist);
        if !lns.is_disjoint(&rns) {
            return false;
        }
    }
    true
}

fn mapping_is_complete(
    all_pairs: &HashSet<i64>,
    mapping: &BTreeMap<Word, Word>,
    lo: &Ortho,
    ro: &Ortho,
) -> bool {
    for (right_location, right_name) in &ro.info {
        if right_location.length() > 1 {
            let mapped = right_location.map_location(mapping);
            let left_name = lo.name_at_location(&mapped);
            if !all_pairs.contains(&ints_to_big_int(left_name, *right_name)) {
                return false;
            }
        }
    }
    true
}

fn mapping_works(
    left_mapping: &[Word],
    fixed_right_hand: &[Word],
    all_pairs: &HashSet<i64>,
) -> bool {
    zip(left_mapping, fixed_right_hand)
        .map(|(try_left, try_right)| ints_to_big_int(*try_left, *try_right))
        .all(|d| all_pairs.contains(&d))
}

fn make_mapping(good_left_hand: &[Word], fixed_right_hand: &[Word]) -> BTreeMap<Word, Word> {
    zip(
        fixed_right_hand.iter().cloned(),
        good_left_hand.iter().cloned(),
    )
    .collect()
}

#[cfg(test)]
mod tests {
    use crate::ortho::Ortho;

    use super::diagonals_do_not_conflict;

    #[test]
    fn diagonals_do_not_conflict_works_on_tricky_inputs() {
        let diagonals_dont_conflict =
            diagonals_do_not_conflict(&Ortho::new(1, 2, 3, 4), &Ortho::new(3, 4, 7, 8));

        assert!(!diagonals_dont_conflict)
    }
}
