use anyhow::Error;
use std::collections::BTreeMap;

use diesel::PgConnection;
use itertools::{zip, Itertools};

use crate::{ortho::Ortho, FailableStringToOrthoVec, FailableStringVecToOrthoVec};

pub(crate) fn over(
    conn: Option<&PgConnection>,
    phrase: Vec<String>,
    ortho_by_origin: FailableStringToOrthoVec,
    ortho_by_hop: FailableStringVecToOrthoVec,
    ortho_by_contents: FailableStringVecToOrthoVec,
    phrase_exists: fn(Option<&PgConnection>, Vec<&String>) -> Result<bool, anyhow::Error>,
) -> Result<Vec<Ortho>, anyhow::Error> {
    let lhs_phrase_head: Vec<&String> = phrase[..phrase.len() - 1].iter().collect();
    let rhs_phrase_head: Vec<&String> = phrase[1..].iter().collect();

    let foo = ortho_by_origin(conn, &phrase[0])?;
    let lhs_by_origin = foo
        .iter()
        .filter(|o| o.origin_has_phrase(&lhs_phrase_head))
        .filter(|o| o.axis_length(&phrase[1]) == (phrase.len() - 2));

    let bar = ortho_by_origin(conn, &phrase[1])?;
    let rhs_by_origin = bar
        .iter()
        .filter(|o| o.origin_has_phrase(&rhs_phrase_head))
        .filter(|o| o.axis_length(&phrase[2]) == (phrase.len() - 2));

    let origin_potential_pairings = Itertools::cartesian_product(lhs_by_origin, rhs_by_origin)
        .filter(|(l, r)| l.get_dims() == r.get_dims())
        .map(|(l, r)| (l, r, phrase[1].clone(), phrase[2].clone()));

    let baz = ortho_by_hop(conn, vec![phrase[0].clone()])?;
    let lhs_by_hop = baz
        .iter()
        .filter(|o| o.hop_has_phrase(&lhs_phrase_head))
        .filter_map(|o| {
            let axis = o.axis_of_change_between_names_for_hop(&phrase[0], &phrase[1]);
            if o.axis_length(&axis) == phrase.len() - 2 {
                Some((o, axis))
            } else {
                None
            }
        });

    let bang = ortho_by_hop(conn, vec![phrase[1].clone()])?;
    let rhs_by_hop = bang
        .iter()
        .filter(|o| o.hop_has_phrase(&rhs_phrase_head))
        .filter_map(|o| {
            let axis = o.axis_of_change_between_names_for_hop(&phrase[1], &phrase[2]);
            if o.axis_length(&axis) == phrase.len() - 2 {
                Some((o, axis))
            } else {
                None
            }
        });

    let hop_potential_pairings = Itertools::cartesian_product(lhs_by_hop, rhs_by_hop)
        .filter(|((l, _lx), (r, _rx))| l.get_dims() == r.get_dims())
        .map(|((l, lx), (r, rx))| (l, r, lx, rx));

    let qux = ortho_by_contents(conn, vec![phrase[0].clone()])?;
    let lhs_by_contents = qux
        .iter()
        .filter(|o| o.contents_has_phrase(&lhs_phrase_head))
        .map(|o| {
            (
                o,
                o.axes_of_change_between_names_for_contents(&phrase[0], &phrase[1]),
            )
        })
        .flat_map(|(o, axs)| axs.into_iter().map(|axis| (o, axis)).collect::<Vec<_>>())
        .filter(|(o, a)| o.axis_length(a) == phrase.len() - 2)
        .map(|(o, a)| (o, a));

    let quux = ortho_by_contents(conn, vec![phrase[1].clone()])?;
    let rhs_by_contents = quux
        .iter()
        .filter(|o| o.contents_has_phrase(&rhs_phrase_head))
        .map(|o| {
            (
                o,
                o.axes_of_change_between_names_for_contents(&phrase[1], &phrase[2]),
            )
        })
        .flat_map(|(o, axs)| axs.into_iter().map(|axis| (o, axis)).collect::<Vec<_>>())
        .filter(|(o, a)| o.axis_length(a) == phrase.len() - 2)
        .map(|(o, a)| (o, a));

    let contents_potential_pairings =
        Itertools::cartesian_product(lhs_by_contents, rhs_by_contents)
            .filter(|((l, _lx), (r, _rx))| l.get_dims() == r.get_dims())
            .map(|((l, lx), (r, rx))| (l, r, lx.to_string(), rx.to_string()));

    let all_inputs = origin_potential_pairings
        .chain(hop_potential_pairings)
        .chain(contents_potential_pairings);

    let mut res = vec![];
    for (lo, ro, lhs, rhs) in all_inputs {
        for answer in attempt_combine_over(conn, phrase_exists, lo, ro, lhs, rhs)? {
            res.push(answer);
        }
    }
    Ok(res)
}

// todo move the network call out
// todo revisit clones here
pub fn attempt_combine_over(
    conn: Option<&PgConnection>,
    phrase_exists: fn(Option<&PgConnection>, Vec<&String>) -> Result<bool, Error>,
    lo: &Ortho,
    ro: &Ortho,
    left_shift_axis: String,  // todo make this a ref
    right_shift_axis: String, // todo make this a ref
) -> Result<Vec<Ortho>, anyhow::Error> {
    let mut ans = vec![];
    let mut lo_hop_set = lo.get_hop();

    lo_hop_set.remove(&left_shift_axis);
    let lo_hop = Vec::from_iter(lo_hop_set.iter().cloned());

    let mut ro_hop_set = ro.get_hop();
    ro_hop_set.remove(&right_shift_axis);

    let fixed_right_hand: Vec<&String> = ro_hop_set.iter().cloned().collect();

    let lo_hop_len = lo_hop.len();
    let left_hand_coordinate_configurations =
        Itertools::permutations(lo_hop.into_iter(), lo_hop_len);

    for left_mapping in left_hand_coordinate_configurations {
        if axis_lengths_match(
            left_mapping.clone(),
            fixed_right_hand.clone(),
            lo.clone(),
            ro.clone(),
        ) {
            let mapping = make_mapping(
                left_mapping,
                fixed_right_hand.clone(),
                &right_shift_axis,
                &left_shift_axis,
            );

            if mapping_works(&mapping, &lo, &ro, &right_shift_axis, &left_shift_axis) {
                let ortho_to_add =
                    Ortho::zip_over(&lo, &ro, &mapping, &right_shift_axis.to_string());

                if phrases_work(
                    phrase_exists,
                    ortho_to_add.clone(),
                    left_shift_axis.to_owned(),
                    conn,
                )? {
                    ans.push(ortho_to_add);
                }
            }
        }
    }

    Ok(ans)
}

fn phrases_work(
    phrase_exists: fn(Option<&PgConnection>, Vec<&String>) -> Result<bool, anyhow::Error>,
    ortho_to_add: Ortho,
    shift_axis: String,
    conn: Option<&PgConnection>,
) -> Result<bool, anyhow::Error> {
    let phrases = ortho_to_add.phrases(&shift_axis);

    for phrase in phrases {
        if !phrase_exists(conn, phrase)? {
            return Ok(false);
        }
    }
    Ok(true)
}

fn axis_lengths_match(
    left_axes: Vec<&String>,
    right_axes: Vec<&String>,
    lo: Ortho,
    ro: Ortho,
) -> bool {
    let left_lengths: Vec<usize> = left_axes.iter().map(|axis| lo.axis_length(axis)).collect();
    let right_lengths: Vec<usize> = right_axes.iter().map(|axis| ro.axis_length(axis)).collect();

    left_lengths == right_lengths
}

fn mapping_works(
    mapping: &BTreeMap<&String, &String>,
    lo: &Ortho,
    ro: &Ortho,
    origin_shift_axis: &str,
    origin_lhs_known_mapping_member: &str,
) -> bool {
    let shift_axis_length = ro.axis_length(origin_shift_axis);

    for (location, name) in ro.to_vec() {
        if location.count_axis(origin_shift_axis) == shift_axis_length {
            continue;
        }
        let mapped = location.map_location(mapping);
        let augmented = mapped.add(origin_lhs_known_mapping_member);
        let name_at_location = lo.name_at_location(&augmented);

        if name != name_at_location {
            return false;
        }
    }
    true
}

fn make_mapping<'a>(
    left_mapping: Vec<&'a String>,
    fixed_right_hand: Vec<&'a String>,
    origin_shift_axis: &'a String,
    origin_lhs_known_mapping_member: &'a String,
) -> std::collections::BTreeMap<&'a String, &'a String> {
    let mut almost: BTreeMap<&String, &String> = zip(fixed_right_hand, left_mapping).collect();
    almost.insert(origin_shift_axis, origin_lhs_known_mapping_member);
    almost
}

#[cfg(test)]
mod tests {
    use diesel::PgConnection;
    use maplit::{btreemap, hashset};

    use crate::ortho::Ortho;

    use super::{axis_lengths_match, over};

    fn fake_phrase_exists(
        _conn: Option<&PgConnection>,
        phrase: Vec<&String>,
    ) -> Result<bool, anyhow::Error> {
        let a = "a".to_owned();
        let b = "b".to_owned();
        let c = "c".to_owned();
        let d = "d".to_owned();
        let e = "e".to_owned();
        let f = "f".to_owned();
        let ps = hashset! {
            vec![&a, &b, &e],
            vec![&c, &d, &f]
        };
        Ok(ps.contains(&phrase))
    }

    fn fake_phrase_exists_two(
        _conn: Option<&PgConnection>,
        phrase: Vec<&String>,
    ) -> Result<bool, anyhow::Error> {
        let a = "a".to_owned();
        let b = "b".to_owned();
        let e = "e".to_owned();
        let ps = hashset! {
            vec![&a, &b, &e]
        };
        Ok(ps.contains(&phrase))
    }

    fn fake_phrase_exists_three(
        _conn: Option<&PgConnection>,
        phrase: Vec<&String>,
    ) -> Result<bool, anyhow::Error> {
        let a = "a".to_owned();
        let b = "b".to_owned();
        let c = "c".to_owned();
        let d = "d".to_owned();
        let e = "e".to_owned();
        let f = "f".to_owned();
        let g = "g".to_owned();
        let h = "h".to_owned();
        let i = "i".to_owned();
        let ps = hashset! {
            vec![&a, &d, &g],
            vec![&b, &e, &h],
            vec![&c, &f, &i]
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

    fn empty_ortho_by_origin(
        _conn: Option<&PgConnection>,
        _o: &str,
    ) -> Result<Vec<Ortho>, anyhow::Error> {
        Ok(vec![])
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

        let e = &"e".to_string();
        let b = &"b".to_string();
        let d = &"d".to_string();
        let c = &"c".to_string();
        let bigger = Ortho::zip_over(
            &l,
            &r,
            &btreemap! {
                e => b,
                d => c
            },
            e,
        );

        let mut pairs = btreemap! { "a" => vec![bigger], "b" => vec![small]};
        Ok(pairs.entry(o).or_default().to_owned())
    }

    fn fake_ortho_by_origin_three(
        _conn: Option<&PgConnection>,
        o: &str,
    ) -> Result<Vec<Ortho>, anyhow::Error> {
        let e = &"e".to_string();
        let b = &"b".to_string();
        let d = &"d".to_string();
        let c = &"c".to_string();
        let l = Ortho::zip_over(
            &Ortho::new(
                "a".to_string(),
                "b".to_string(),
                "c".to_string(),
                "d".to_string(),
            ),
            &Ortho::new(
                "b".to_string(),
                "e".to_string(),
                "d".to_string(),
                "f".to_string(),
            ),
            &btreemap! {
                e => b,
                d => c
            },
            e,
        );

        // a b e   b e + e g
        // c d f   d f   f h

        let g = &"g".to_string();
        let f = &"f".to_string();
        let r = Ortho::zip_over(
            &Ortho::new(
                "b".to_string(),
                "e".to_string(),
                "d".to_string(),
                "f".to_string(),
            ),
            &Ortho::new(
                "e".to_string(),
                "g".to_string(),
                "f".to_string(),
                "h".to_string(),
            ),
            &btreemap! {
                g => e,
                f => d
            },
            g,
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
        let e = &"e".to_string();
        let b = &"b".to_string();
        let d = &"d".to_string();
        let c = &"c".to_string();
        let abecdf = Ortho::zip_over(
            &abcd,
            &bedf,
            &btreemap! {
                e => b,
                d => c
            },
            e,
        );

        // c d   d f
        // h i   i j
        let f = &"f".to_string();
        let i = &"i".to_string();
        let h = &"h".to_string();
        let cdfhij = Ortho::zip_over(
            &cdhi,
            &dfij,
            &btreemap! {
                f => d,
                i => h
            },
            f,
        );

        // a b e
        // c d f

        // c d f
        // h i j
        let abecdfhij = Ortho::zip_over(
            &abecdf,
            &cdfhij,
            &btreemap! {
                d => b,
                h => c
            },
            h,
        );

        // b e  e b
        // d f  f e
        let bebdfe = Ortho::zip_over(
            &bedf,
            &ebfe,
            &btreemap! {
                b => e,
                f => d
            },
            b,
        );

        // d f   f e
        // i j   j g
        let j = &"j".to_string();
        let dfeijg = Ortho::zip_over(
            &dfij,
            &fejg,
            &btreemap! {
                e => f,
                j => i
            },
            e,
        );

        //  b e b
        //  d f e

        //  d f e
        //  i j g
        let bebdfeijg = Ortho::zip_over(
            &bebdfe,
            &dfeijg,
            &btreemap! {
                f => e,
                i => d
            },
            i,
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

    fn fake_ortho_by_contents(
        _conn: Option<&PgConnection>,
        o: Vec<String>,
    ) -> Result<Vec<Ortho>, anyhow::Error> {
        let mut ans = vec![];

        // a b c
        // d e f

        // d e f
        // g h i

        let abde = Ortho::new(
            "a".to_string(),
            "b".to_string(),
            "d".to_string(),
            "e".to_string(),
        );

        let bcef = Ortho::new(
            "b".to_string(),
            "c".to_string(),
            "e".to_string(),
            "f".to_string(),
        );

        let degh = Ortho::new(
            "d".to_string(),
            "e".to_string(),
            "g".to_string(),
            "h".to_string(),
        );

        let efhi = Ortho::new(
            "e".to_string(),
            "f".to_string(),
            "h".to_string(),
            "i".to_string(),
        );

        let c = &"c".to_string();
        let b = &"b".to_string();
        let e = &"e".to_string();
        let d = &"d".to_string();
        let abcdef = Ortho::zip_over(
            &abde,
            &bcef,
            &btreemap! {
                c => b,
                e => d
            },
            c,
        );

        let f = &"f".to_string();
        let h = &"h".to_string();
        let g = &"g".to_string();
        let defghi = Ortho::zip_over(
            &degh,
            &efhi,
            &btreemap! {
                f => e,
                h => g
            },
            f,
        );

        if o.contains(&"c".to_string())
            || o.contains(&"e".to_string())
            || o.contains(&"f".to_string())
        {
            ans.push(abcdef);
        }

        if o.contains(&"f".to_string())
            || o.contains(&"h".to_string())
            || o.contains(&"i".to_string())
        {
            ans.push(defghi);
        }

        Ok(ans)
    }

    fn fake_ortho_by_hop(
        _conn: Option<&PgConnection>,
        o: Vec<String>,
    ) -> Result<Vec<Ortho>, anyhow::Error> {
        let mut ans = vec![];

        if o.contains(&"b".to_string()) {
            ans.push(Ortho::new(
                "a".to_string(),
                "b".to_string(),
                "c".to_string(),
                "d".to_string(),
            ))
        }

        if o.contains(&"c".to_string()) {
            ans.push(Ortho::new(
                "a".to_string(),
                "b".to_string(),
                "c".to_string(),
                "d".to_string(),
            ))
        }

        if o.contains(&"e".to_string()) {
            ans.push(Ortho::new(
                "b".to_string(),
                "e".to_string(),
                "d".to_string(),
                "f".to_string(),
            ))
        }

        if o.contains(&"d".to_string()) {
            ans.push(Ortho::new(
                "b".to_string(),
                "e".to_string(),
                "d".to_string(),
                "f".to_string(),
            ))
        }

        Ok(ans)
    }

    fn empty_ortho_by_contents(
        _conn: Option<&PgConnection>,
        _o: Vec<String>,
    ) -> Result<Vec<Ortho>, anyhow::Error> {
        Ok(vec![])
    }

    #[test]
    fn over_by_origin() {
        // a b | b e    =   a b e
        // c d | d f        c d f

        let e = &"e".to_string();
        let b = &"b".to_string();
        let d = &"d".to_string();
        let c = &"c".to_string();
        let expected = Ortho::zip_over(
            &Ortho::new(
                "a".to_string(),
                "b".to_string(),
                "c".to_string(),
                "d".to_string(),
            ),
            &Ortho::new(
                "b".to_string(),
                "e".to_string(),
                "d".to_string(),
                "f".to_string(),
            ),
            &btreemap! {
                e => b,
                d => c
            },
            e,
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

    #[test]
    fn axis_lengths_can_match() {
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

        let dfij = Ortho::new(
            "d".to_string(),
            "f".to_string(),
            "i".to_string(),
            "j".to_string(),
        );

        // b e  e b
        // d f  f e
        let b = &"b".to_string();
        let f = &"f".to_string();
        let d = &"d".to_string();
        let e = &"e".to_string();
        let bebdfe = Ortho::zip_over(
            &bedf,
            &ebfe,
            &btreemap! {
                b => e,
                f => d
            },
            b,
        );

        // d f   f e
        // i j   j g
        let e = &"e".to_string();
        let f = &"f".to_string();
        let j = &"j".to_string();
        let i = &"i".to_string();
        let dfeijg = Ortho::zip_over(
            &dfij,
            &fejg,
            &btreemap! {
                e => f,
                j => i
            },
            e,
        );

        // b e b   d f e
        // d f e   i j g
        let yes = axis_lengths_match(
            vec![&"e".to_string(), &"d".to_string()],
            vec![&"f".to_string(), &"i".to_string()],
            bebdfe.clone(),
            dfeijg.clone(),
        );

        let no = axis_lengths_match(
            vec![&"d".to_string(), &"e".to_string()],
            vec![&"f".to_string(), &"i".to_string()],
            bebdfe,
            dfeijg,
        );

        assert!(yes);
        assert!(!no);
    }

    #[test]
    fn over_by_origin_filters_if_a_phrase_is_missing_from_db() {
        let actual = over(
            None,
            vec!["a".to_owned(), "b".to_owned(), "e".to_owned()],
            fake_ortho_by_origin,
            empty_ortho_by_hop,
            empty_ortho_by_contents,
            fake_phrase_exists_two,
        )
        .unwrap();

        assert_eq!(actual.len(), 0);
    }

    #[test]
    fn over_by_hop() {
        // a b | b e    =   a b e
        // c d | d f        c d f
        let e = &"e".to_string();
        let b = &"b".to_string();
        let d = &"d".to_string();
        let c = &"c".to_string();
        let expected = Ortho::zip_over(
            &Ortho::new(
                "a".to_string(),
                "b".to_string(),
                "c".to_string(),
                "d".to_string(),
            ),
            &Ortho::new(
                "b".to_string(),
                "e".to_string(),
                "d".to_string(),
                "f".to_string(),
            ),
            &btreemap! {
                e => b,
                d => c
            },
            e,
        );

        let actual = over(
            None,
            vec!["c".to_owned(), "d".to_owned(), "f".to_owned()],
            empty_ortho_by_origin,
            fake_ortho_by_hop,
            empty_ortho_by_contents,
            fake_phrase_exists,
        )
        .unwrap();

        assert_eq!(vec![expected], actual);
    }

    #[test]
    fn over_by_contents() {
        // a b c
        // d e f

        // d e f
        // g h i

        // a b c
        // d e f
        // g h i

        // phrase: c f i

        let abde = Ortho::new(
            "a".to_string(),
            "b".to_string(),
            "d".to_string(),
            "e".to_string(),
        );

        let bcef = Ortho::new(
            "b".to_string(),
            "c".to_string(),
            "e".to_string(),
            "f".to_string(),
        );

        let degh = Ortho::new(
            "d".to_string(),
            "e".to_string(),
            "g".to_string(),
            "h".to_string(),
        );

        let efhi = Ortho::new(
            "e".to_string(),
            "f".to_string(),
            "h".to_string(),
            "i".to_string(),
        );

        let e = &"e".to_string();
        let b = &"b".to_string();
        let d = &"d".to_string();
        let c = &"c".to_string();
        let abcdef = Ortho::zip_over(
            &abde,
            &bcef,
            &btreemap! {
                c => b,
                e => d
            },
            c,
        );

        let f = &"f".to_string();
        let h = &"h".to_string();
        let g = &"g".to_string();
        let defghi = Ortho::zip_over(
            &degh,
            &efhi,
            &btreemap! {
                f => e,
                h => g
            },
            f,
        );

        let expected = Ortho::zip_over(
            &abcdef,
            &defghi,
            &btreemap! {
                e => b,
                g => d
            },
            g,
        );

        let actual = over(
            None,
            vec!["c".to_owned(), "f".to_owned(), "i".to_owned()],
            empty_ortho_by_origin,
            empty_ortho_by_hop,
            fake_ortho_by_contents,
            fake_phrase_exists_three,
        )
        .unwrap();

        assert_eq!(vec![expected], actual);
    }
}
