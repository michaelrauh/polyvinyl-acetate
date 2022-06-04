use crate::models::{NewOrthotope, Orthotope};
use crate::ortho::Ortho;
use crate::pair_todo_handler;
use crate::schema::orthotopes;
use crate::schema::pairs::{first_word, second_word, table as pairs};
use diesel::dsl::exists;
use diesel::query_dsl::filter_dsl::FilterDsl;
use diesel::{BoolExpressionMethods, ExpressionMethods, PgConnection, RunQueryDsl};
use itertools::{zip, Itertools};
use std::collections::BTreeMap;

pub type FailableBoolOnPair =
    fn(Option<&PgConnection>, try_left: &str, try_right: &str) -> Result<bool, anyhow::Error>;

pub fn attempt_up(
    conn: Option<&PgConnection>,
    pair_checker: FailableBoolOnPair,
    ans: &mut Vec<Ortho>,
    lo: Ortho,
    ro: Ortho,
) -> Result<(), anyhow::Error> {
    if lo.get_dims() == ro.get_dims() {
        let lo_hop = lo.get_hop();
        let left_hand_coordinate_configurations =
            Itertools::permutations(lo_hop.iter(), lo_hop.len());
        let fixed_right_hand: Vec<String> = ro.get_hop().into_iter().collect();
        for left_mapping in left_hand_coordinate_configurations {
            if mapping_works(
                conn,
                pair_checker,
                left_mapping.clone(),
                fixed_right_hand.clone(),
            )? {
                let mapping = make_mapping(left_mapping, fixed_right_hand.clone());
                if mapping_is_complete(conn, pair_checker, mapping.clone(), lo.clone(), ro.clone())?
                    && diagonals_do_not_conflict(lo.clone(), ro.clone())
                {
                    ans.push(Ortho::zip_up(lo.clone(), ro.clone(), mapping));
                }
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
    conn: Option<&PgConnection>,
    pair_checker: FailableBoolOnPair,
    left_mapping: Vec<&String>,
    fixed_right_hand: Vec<String>,
) -> Result<bool, anyhow::Error> {
    for (try_left, try_right) in zip(left_mapping, fixed_right_hand) {
        if !pair_checker(conn, try_left, &try_right)? {
            return Ok(false);
        }
    }
    Ok(true)
}

fn make_mapping(
    good_left_hand: Vec<&String>,
    fixed_right_hand: Vec<String>,
) -> BTreeMap<String, String> {
    let left_hand_owned: Vec<String> = good_left_hand.iter().map(|x| x.to_string()).collect();
    zip(fixed_right_hand, left_hand_owned).collect()
}

pub fn get_ortho_by_origin(
    conn: Option<&PgConnection>,
    o: &str,
) -> Result<Vec<Ortho>, anyhow::Error> {
    use crate::schema;
    use crate::schema::orthotopes::{origin, table as orthotopes};
    use diesel::query_dsl::methods::SelectDsl;
    let results: Vec<Orthotope> = orthotopes
        .filter(origin.eq(o))
        .select(schema::orthotopes::all_columns)
        .load(conn.expect("don't use test connections in production"))?;

    let res: Vec<Ortho> = results
        .iter()
        .map(|x| bincode::deserialize(&x.information).expect("deserialization should succeed"))
        .collect();
    Ok(res)
}

pub fn pair_exists(
    conn: Option<&PgConnection>,
    try_left: &str,
    try_right: &str,
) -> Result<bool, anyhow::Error> {
    let res: bool = diesel::select(exists(
        pairs.filter(first_word.eq(try_left).and(second_word.eq(try_right))),
    ))
    .get_result(conn.expect("don't use the test connection"))?;

    Ok(res)
}

pub fn insert_orthotopes(
    conn: &PgConnection,
    new_orthos: &[NewOrthotope],
) -> Result<Vec<Orthotope>, diesel::result::Error> {
    diesel::insert_into(orthotopes::table)
        .values(new_orthos)
        .on_conflict_do_nothing()
        .get_results(conn)
}

pub fn ortho_to_orthotope(ortho: &Ortho) -> NewOrthotope {
    let information = bincode::serialize(&ortho).expect("serialization should work");
    let origin = ortho.get_origin();
    let hop = Vec::from_iter(ortho.get_hop());
    let contents = Vec::from_iter(ortho.get_contents());
    let info_hash = pair_todo_handler::data_vec_to_signed_int(&information);
    NewOrthotope {
        information,
        origin,
        hop,
        contents,
        info_hash,
    }
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