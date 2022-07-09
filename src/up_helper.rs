use crate::ortho::Ortho;
use crate::schema::pairs::{first_word, pair_hash, second_word, table as pairs};
use crate::vec_of_strings_to_signed_int;
use diesel::dsl::exists;
use diesel::query_dsl::filter_dsl::FilterDsl;
use diesel::{BoolExpressionMethods, ExpressionMethods, PgConnection, RunQueryDsl};
use itertools::{zip, Itertools};
use std::collections::{BTreeMap, HashSet};

pub type FailableBoolOnPair =
    fn(Option<&PgConnection>, try_left: &str, try_right: &str) -> Result<bool, anyhow::Error>;

pub fn attempt_up(
    conn: Option<&PgConnection>,
    pair_checker: FailableBoolOnPair,
    all_pairs: HashSet<i64>,
    ans: &mut Vec<Ortho>,
    lo: Ortho,
    ro: Ortho,
) -> Result<(), anyhow::Error> {
    let lo_hop = lo.get_hop();
    let left_hand_coordinate_configurations = Itertools::permutations(lo_hop.iter(), lo_hop.len());
    let fixed_right_hand: Vec<String> = ro.get_hop().into_iter().collect();
    for left_mapping in left_hand_coordinate_configurations {
        if mapping_works(
            left_mapping.clone(),
            fixed_right_hand.clone(),
            all_pairs.clone()
        )? {
            let mapping = make_mapping(left_mapping, fixed_right_hand.clone());
            if mapping_is_complete(conn, pair_checker, mapping.clone(), lo.clone(), ro.clone())?
                && diagonals_do_not_conflict(lo.clone(), ro.clone())
            {
                ans.push(Ortho::zip_up(lo.clone(), ro.clone(), mapping));
            }
        }
    }
    Ok(())
}

fn diagonals_do_not_conflict(lo: Ortho, ro: Ortho) -> bool {
    for dist in 0..lo.get_dimensionality() + 1 {
        let lns = lo.get_names_at_distance(dist);
        let rns = ro.get_names_at_distance(dist);

        if !lns.is_disjoint(&rns) {
            return false;
        }
    }
    true
}

fn mapping_is_complete(
    conn: Option<&PgConnection>,
    pair_checker: FailableBoolOnPair,
    mapping: BTreeMap<String, String>,
    lo: Ortho,
    ro: Ortho,
) -> Result<bool, anyhow::Error> {
    for (right_location, right_name) in ro.info {
        if right_location.length() > 1 {
            let mapped = right_location.map_location(mapping.clone());
            let left_name = lo.name_at_location(mapped);
            if !pair_checker(conn, &left_name, &right_name)? {
                return Ok(false);
            }
        }
    }
    Ok(true)
}

fn mapping_works(
    left_mapping: Vec<&String>,
    fixed_right_hand: Vec<String>,
    all_pairs: HashSet<i64>
) -> Result<bool, anyhow::Error> {
    let desired = zip(left_mapping, fixed_right_hand).map(|(try_left, try_right)| vec_of_strings_to_signed_int(vec![try_left.to_string(), try_right])).collect::<HashSet<_>>();
    Ok(desired.is_subset(&all_pairs))
}

fn make_mapping(
    good_left_hand: Vec<&String>,
    fixed_right_hand: Vec<String>,
) -> BTreeMap<String, String> {
    let left_hand_owned: Vec<String> = good_left_hand.iter().map(|x| x.to_string()).collect();
    zip(fixed_right_hand, left_hand_owned).collect()
}

pub fn pair_exists(
    conn: Option<&PgConnection>,
    try_left: &str,
    try_right: &str,
) -> Result<bool, anyhow::Error> {
    let res: bool = diesel::select(exists(pairs.filter(pair_hash.eq(
        vec_of_strings_to_signed_int(vec![try_left.to_string(), try_right.to_string()]),
    ))))
    .get_result(conn.expect("don't use the test connection"))?;

    Ok(res)
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
