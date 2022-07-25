use crate::{ortho::Ortho, string_refs_to_signed_int};

use itertools::{zip, Itertools};

use std::collections::{BTreeMap, HashSet};

pub fn attempt_up(all_pairs: &HashSet<i64>, lo: &Ortho, ro: &Ortho) -> Vec<Ortho> {
    let lo_hop = lo.get_hop();
    let lo_hop_len = lo_hop.len();
    let left_hand_coordinate_configurations =
        Itertools::permutations(lo_hop.into_iter(), lo_hop_len);
    let fixed_right_hand: Vec<&String> = ro.get_hop().into_iter().collect();
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
    mapping: &BTreeMap<&String, &String>,
    lo: &Ortho,
    ro: &Ortho,
) -> bool {
    for (right_location, right_name) in &ro.info {
        if right_location.length() > 1 {
            let mapped = right_location.map_location_lean(mapping);
            let left_name = lo.name_at_location(mapped);
            if !all_pairs.contains(&string_refs_to_signed_int(&left_name, right_name)) {
                return false;
            }
        }
    }
    true
}

fn mapping_works(
    left_mapping: &[&String],
    fixed_right_hand: &[&String],
    all_pairs: &HashSet<i64>,
) -> bool {
    zip(left_mapping, fixed_right_hand)
        .map(|(try_left, try_right)| string_refs_to_signed_int(try_left, try_right))
        .all(|d| all_pairs.contains(&d))
}

fn make_mapping<'a>(
    good_left_hand: &[&'a String],
    fixed_right_hand: &'a [&'a String],
) -> BTreeMap<&'a String, &'a String> {
    zip(
        fixed_right_hand.iter().cloned(),
        good_left_hand.iter().cloned(),
    )
    .collect()
}

pub fn filter_base(orthos: Vec<Ortho>) -> Vec<Ortho> {
    orthos.into_iter().filter(|o| o.is_base()).collect()
}

pub fn make_potential_pairings(
    left_orthos_by_origin: Vec<Ortho>,
    right_orthos_by_origin: Vec<Ortho>,
) -> Vec<(Ortho, Ortho)> {
    let potential_pairings_by_origin: Vec<(Ortho, Ortho)> = Itertools::cartesian_product(
        left_orthos_by_origin.iter().cloned(),
        right_orthos_by_origin.iter().cloned(),
    )
    .collect();
    potential_pairings_by_origin
}

#[cfg(test)]
mod tests {
    use crate::ortho::Ortho;

    use super::diagonals_do_not_conflict;

    #[test]
    fn diagonals_do_not_conflict_works_on_tricky_inputs() {
        let diagonals_dont_conflict = diagonals_do_not_conflict(
            &Ortho::new(
                "a".to_string(),
                "b".to_string(),
                "c".to_string(),
                "d".to_string(),
            ),
            &Ortho::new(
                "c".to_string(),
                "d".to_string(),
                "g".to_string(),
                "h".to_string(),
            ),
        );

        assert!(!diagonals_dont_conflict)
    }
}
