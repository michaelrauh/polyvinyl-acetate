use std::collections::BTreeMap;

use diesel::PgConnection;
use itertools::{zip, Itertools};
use maplit::hashset;

use crate::{
    ortho::{Location, Ortho},
    up_helper::make_potential_pairings,
    FailableStringToOrthoVec, FailableStringVecToOrthoVec,
};

pub(crate) fn over(
    conn: Option<&PgConnection>,
    phrase: Vec<String>,
    ortho_by_origin: FailableStringToOrthoVec,
    ortho_by_hop: FailableStringVecToOrthoVec,
    ortho_by_contents: FailableStringVecToOrthoVec,
    phrase_exists: fn(Option<&PgConnection>, Vec<String>) -> Result<bool, anyhow::Error>,
) -> Result<Vec<Ortho>, anyhow::Error> {
    let mut ans = vec![];
    if phrase.len() < 3 {
        return Ok(ans);
    }

    let lhs_phrase_head: Vec<String> = phrase[..phrase.len() - 1].to_vec();
    let rhs_phrase_head: Vec<String> = phrase[1..].to_vec();
    let origin_shift_axis = &phrase[2].to_owned(); // by origin shift axis must be the third word, as it follows the origin of the rhs. This logic does not apply to hop or contents
    let origin_lhs_known_mapping_member = (&phrase[1]).to_owned(); // origin only logic

    let lhs_by_origin: Vec<Ortho> = ortho_by_origin(conn, &phrase[0])?
        .iter()
        .filter(|o| o.contains_phrase(lhs_phrase_head.clone()))
        .filter(|o| o.get_hop().contains(&phrase[1])) // for origin, hop must have the second word as it is starting in the corner. Origin only logic
        .filter(|o| o.axis_length(&origin_lhs_known_mapping_member) == (phrase.len() - 2)) // offset is two as it doesn't count origin or extra word. origin only logic
        .cloned()
        .collect();

    let rhs_by_origin: Vec<Ortho> = ortho_by_origin(conn, &phrase[1])?
        .iter()
        .filter(|o| o.contains_phrase(rhs_phrase_head.clone()))
        .filter(|o| o.axis_length(origin_shift_axis) == (phrase.len() - 2)) // offset is two as it doesn't count origin or extra word. origin only logic
        .filter(|o| {
            o.axis_has_phrase(
                &rhs_phrase_head,
                Location::default(),
                origin_shift_axis.to_string(),
            ) // origin only logic
        })
        .cloned()
        .collect();

    let potential_pairings: Vec<(Ortho, Ortho)> =
        make_potential_pairings(lhs_by_origin, rhs_by_origin)
            .iter()
            .filter(|(l, r)| l.get_dims() == r.get_dims())
            .cloned()
            .collect();

    for (lo, ro) in potential_pairings.clone() {
        let lo_hop: Vec<String> = lo
            .get_hop()
            .difference(&hashset! {origin_lhs_known_mapping_member.clone()})
            .cloned()
            .collect();
        let fixed_right_hand: Vec<String> = ro
            .get_hop()
            .difference(&hashset! {origin_shift_axis.to_owned()})
            .cloned()
            .collect();

        let left_hand_coordinate_configurations =
            Itertools::permutations(lo_hop.iter(), lo_hop.len());

        for left_mapping in left_hand_coordinate_configurations {
            let mapping = make_mapping(
                left_mapping,
                fixed_right_hand.clone(),
                origin_shift_axis,
                origin_lhs_known_mapping_member.clone(),
            );

            if mapping_works(
                mapping.clone(),
                lo.clone(),
                ro.clone(),
                origin_shift_axis,
                &origin_lhs_known_mapping_member,
            ) {
                ans.push(Ortho::zip_over(
                    lo.clone(),
                    ro.clone(),
                    mapping.clone(),
                    origin_shift_axis.to_string(),
                ));
            }
        }
    }

    Ok(ans)
}

fn mapping_works(
    mapping: BTreeMap<String, String>,
    lo: Ortho,
    ro: Ortho,
    origin_shift_axis: &str,
    origin_lhs_known_mapping_member: &str,
) -> bool {
    let shift_axis_length: usize = ro.axis_length(origin_shift_axis);

    for (location, name) in ro.to_vec() {
        if location.count_axis(origin_shift_axis) == shift_axis_length {
            continue;
        }
        let mapped = location.map_location(mapping.clone());
        let augmented = mapped.add(origin_lhs_known_mapping_member.to_string());
        let name_at_location = lo.name_at_location(augmented);

        if name != name_at_location {
            return false;
        }
    }
    true
}

fn make_mapping(
    left_mapping: Vec<&String>,
    fixed_right_hand: Vec<String>,
    origin_shift_axis: &str,
    origin_lhs_known_mapping_member: String,
) -> std::collections::BTreeMap<String, String> {
    let left_hand_owned: Vec<String> = left_mapping.iter().map(|x| x.to_string()).collect();
    let mut almost: BTreeMap<String, String> = zip(fixed_right_hand, left_hand_owned).collect();
    almost.insert(
        origin_shift_axis.to_string(),
        origin_lhs_known_mapping_member,
    );
    almost
}

#[cfg(test)]
mod tests {
    use diesel::PgConnection;
    use maplit::{btreemap, hashset};

    use crate::ortho::Ortho;

    use super::over;

    fn fake_phrase_exists(
        _conn: Option<&PgConnection>,
        phrase: Vec<String>,
    ) -> Result<bool, anyhow::Error> {
        let ps = hashset! {
            vec!["a".to_owned(), "b".to_owned(), "e".to_owned()],
            vec!["c".to_owned(), "d".to_owned(), "f".to_owned()]
        };
        Ok(ps.contains(&phrase))
    }

    fn fake_ortho_by_origin(
        _conn: Option<&PgConnection>,
        o: &str,
    ) -> Result<Vec<Ortho>, anyhow::Error> {
        let mut pairs = btreemap! { "a" => vec![Ortho::new(
            "a".to_string(),
            "b".to_string(),
            "c".to_string(),
            "d".to_string(),
        )], "b" => vec![Ortho::new(
            "b".to_string(),
            "e".to_string(),
            "d".to_string(),
            "f".to_string(),
        )]};
        Ok(pairs.entry(o).or_default().to_owned())
    }

    fn fake_ortho_by_origin_two(
        _conn: Option<&PgConnection>,
        o: &str,
    ) -> Result<Vec<Ortho>, anyhow::Error> {
        let small = Ortho::new(
            "b".to_string(),
            "d".to_string(),
            "e".to_string(),
            "y".to_string(),
        );

        // a b e
        // c d f

        // b e
        // d y
        let l = Ortho::new(
            "a".to_string(),
            "b".to_string(),
            "c".to_string(),
            "d".to_string(),
        );

        let r = Ortho::new(
            "b".to_string(),
            "e".to_string(),
            "d".to_string(),
            "f".to_string(),
        );
        let bigger = Ortho::zip_over(
            l,
            r,
            btreemap! {
                "e".to_string() => "b".to_string(),
                "d".to_string() => "c".to_string()
            },
            "e".to_string(),
        );

        let mut pairs = btreemap! { "a" => vec![bigger], "b" => vec![small]};
        Ok(pairs.entry(o).or_default().to_owned())
    }

    fn fake_ortho_by_origin_three(
        _conn: Option<&PgConnection>,
        o: &str,
    ) -> Result<Vec<Ortho>, anyhow::Error> {
        let l = Ortho::zip_over(
            Ortho::new(
                "a".to_string(),
                "b".to_string(),
                "c".to_string(),
                "d".to_string(),
            ),
            Ortho::new(
                "b".to_string(),
                "e".to_string(),
                "d".to_string(),
                "f".to_string(),
            ),
            btreemap! {
                "e".to_string() => "b".to_string(),
                "d".to_string() => "c".to_string()
            },
            "e".to_string(),
        );

        // a b e   b e + e g
        // c d f   d f   f h

        let r = Ortho::zip_over(
            Ortho::new(
                "b".to_string(),
                "e".to_string(),
                "d".to_string(),
                "f".to_string(),
            ),
            Ortho::new(
                "e".to_string(),
                "g".to_string(),
                "f".to_string(),
                "h".to_string(),
            ),
            btreemap! {
                "g".to_string() => "e".to_string(),
                "f".to_string() => "d".to_string()
            },
            "g".to_string(),
        );

        let mut pairs = btreemap! { "a" => vec![l], "b" => vec![r]};
        Ok(pairs.entry(o).or_default().to_owned())
    }

    fn fake_ortho_by_origin_four(
        _conn: Option<&PgConnection>,
        o: &str,
    ) -> Result<Vec<Ortho>, anyhow::Error> {
        // a b e   b e b
        // c d f   d f e
        // h i j   i j g

        // phrase: a b e g
        // LHS: a b e
        // RHS: b e g

        let abcd = Ortho::new(
            "a".to_string(),
            "b".to_string(),
            "c".to_string(),
            "d".to_string(),
        );

        let fejg = Ortho::new(
            "f".to_string(),
            "e".to_string(),
            "j".to_string(),
            "g".to_string(),
        );

        let ebfe = Ortho::new(
            "e".to_string(),
            "b".to_string(),
            "f".to_string(),
            "e".to_string(),
        );

        let bedf = Ortho::new(
            "b".to_string(),
            "e".to_string(),
            "d".to_string(),
            "f".to_string(),
        );

        let cdhi = Ortho::new(
            "c".to_string(),
            "d".to_string(),
            "h".to_string(),
            "i".to_string(),
        );

        let dfij = Ortho::new(
            "d".to_string(),
            "f".to_string(),
            "i".to_string(),
            "j".to_string(),
        );

        // a b  b e
        // c d  d f
        let abecdf = Ortho::zip_over(
            abcd,
            bedf.clone(),
            btreemap! {
                "e".to_string() => "b".to_string(),
                "d".to_string() => "c".to_string()
            },
            "e".to_string(),
        );

        // c d   d f
        // h i   i j
        let cdfhij = Ortho::zip_over(
            cdhi,
            dfij.clone(),
            btreemap! {
                "f".to_string() => "d".to_string(),
                "i".to_string() => "h".to_string()
            },
            "f".to_string(),
        );

        // a b e
        // c d f

        // c d f
        // h i j
        let abecdfhij = Ortho::zip_over(
            abecdf,
            cdfhij,
            btreemap! {
                "d".to_string() => "b".to_string(),
                "h".to_string() => "c".to_string()
            },
            "h".to_string(),
        );

        // b e  e b
        // d f  f e
        let bebdfe = Ortho::zip_over(
            bedf,
            ebfe,
            btreemap! {
                "b".to_string() => "e".to_string(),
                "f".to_string() => "d".to_string()
            },
            "b".to_string(),
        );

        // d f   f e
        // i j   j g
        let dfeijg = Ortho::zip_over(
            dfij,
            fejg,
            btreemap! {
                "e".to_string() => "f".to_string(),
                "j".to_string() => "i".to_string()
            },
            "e".to_string(),
        );

        //  b e b
        //  d f e

        //  d f e
        //  i j g
        let bebdfeijg = Ortho::zip_over(
            bebdfe,
            dfeijg,
            btreemap! {
                "f".to_string() => "e".to_string(),
                "i".to_string() => "d".to_string()
            },
            "i".to_string(),
        );

        let mut pairs = btreemap! { "a" => vec![abecdfhij], "b" => vec![bebdfeijg]};
        Ok(pairs.entry(o).or_default().to_owned())
    }

    fn empty_ortho_by_hop(
        _conn: Option<&PgConnection>,
        _o: Vec<String>,
    ) -> Result<Vec<Ortho>, anyhow::Error> {
        Ok(vec![])
    }

    fn empty_ortho_by_contents(
        _conn: Option<&PgConnection>,
        _o: Vec<String>,
    ) -> Result<Vec<Ortho>, anyhow::Error> {
        Ok(vec![])
    }

    #[test]
    fn over_with_phrase_of_length_two_or_less_is_empty() {
        let actual = over(
            None,
            vec!["a".to_owned(), "b".to_owned()],
            fake_ortho_by_origin,
            empty_ortho_by_hop,
            empty_ortho_by_contents,
            fake_phrase_exists,
        )
        .unwrap();

        assert_eq!(actual, vec![])
    }

    #[test]
    fn over_by_origin() {
        // a b | b e    =   a b e
        // c d | d f        c d f

        let expected = Ortho::zip_over(
            Ortho::new(
                "a".to_string(),
                "b".to_string(),
                "c".to_string(),
                "d".to_string(),
            ),
            Ortho::new(
                "b".to_string(),
                "e".to_string(),
                "d".to_string(),
                "f".to_string(),
            ),
            btreemap! {
                "e".to_string() => "b".to_string(),
                "d".to_string() => "c".to_string()
            },
            "e".to_string(),
        );

        let actual = over(
            None,
            vec!["a".to_owned(), "b".to_owned(), "e".to_owned()],
            fake_ortho_by_origin,
            empty_ortho_by_hop,
            empty_ortho_by_contents,
            fake_phrase_exists,
        )
        .unwrap();

        assert_eq!(vec![expected], actual);
    }

    #[test]
    fn over_filters_mismatched_dims() {
        let actual = over(
            None,
            vec!["a".to_owned(), "b".to_owned(), "e".to_owned()],
            fake_ortho_by_origin_two,
            empty_ortho_by_hop,
            empty_ortho_by_contents,
            fake_phrase_exists,
        )
        .unwrap();

        assert_eq!(actual.len(), 0);
    }

    #[test]
    fn over_filters_shift_axis_is_wrong_length() {
        let actual = over(
            None,
            vec!["a".to_owned(), "b".to_owned(), "e".to_owned()],
            fake_ortho_by_origin_three,
            empty_ortho_by_hop,
            empty_ortho_by_contents,
            fake_phrase_exists,
        )
        .unwrap();

        assert_eq!(actual.len(), 0);
    }

    #[test]
    fn over_filters_if_the_phrase_wont_result() {
        let actual = over(
            None,
            vec![
                "a".to_owned(),
                "b".to_owned(),
                "e".to_owned(),
                "g".to_owned(),
            ],
            fake_ortho_by_origin_four,
            empty_ortho_by_hop,
            empty_ortho_by_contents,
            fake_phrase_exists,
        )
        .unwrap();

        assert_eq!(actual.len(), 0);
    }
}
// for hop and contents (and for origin for that matter, but this is a slower way to find it), shift axis is the axis that increases in the rhs while traversing the phrase

// filters axis mappings if the axes are not the same length
// filters by all required phrases are in db - you can extract the right phrases by starting with everything not having the shift axis, and then traverse that axis.
// by hop
// by contents
// integrated test
