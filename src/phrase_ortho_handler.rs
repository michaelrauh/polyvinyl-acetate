use anyhow::Error;
use std::collections::{BTreeMap, HashSet};

use diesel::PgConnection;
use itertools::{zip, Itertools};

use crate::{ortho::Ortho, FailableWordToOrthoVec, FailableWordVecToOrthoVec, Word, vec_of_words_to_big_int};

pub(crate) fn over_by_origin(
    conn: Option<&PgConnection>,
    phrase: Vec<Word>,
    ortho_by_origin: FailableWordToOrthoVec,
    phrase_exists_db_filter: fn(
        Option<&PgConnection>,
        HashSet<i64>,
        HashSet<i64>,
    ) -> Result<HashSet<i64>, anyhow::Error>,
) -> Result<Vec<Ortho>, anyhow::Error> {
    let lhs_phrase_head = &phrase[..phrase.len() - 1];
    let rhs_phrase_head = &phrase[1..];

    let orthos_by_origin_left = ortho_by_origin(conn, phrase[0])?;
    let lhs_by_origin = orthos_by_origin_left
        .iter()
        .filter(|o| o.origin_has_phrase(lhs_phrase_head))
        .filter(|o| o.axis_length(phrase[1]) == (phrase.len() - 2));

    let orthos_by_origin_right = ortho_by_origin(conn, phrase[1])?;
    let rhs_by_origin = orthos_by_origin_right
        .iter()
        .filter(|o| o.origin_has_phrase(rhs_phrase_head))
        .filter(|o| o.axis_length(phrase[2]) == (phrase.len() - 2));

    let origin_potential_pairings = Itertools::cartesian_product(lhs_by_origin, rhs_by_origin)
        .filter(|(l, r)| l.get_dims() == r.get_dims())
        .map(|(l, r)| (l, r, phrase[1], phrase[2]));

    let all_inputs = origin_potential_pairings.clone();

    let all_phrase_heads_left: HashSet<i64> = origin_potential_pairings.clone().flat_map(|(l, _r, sl, _sr)| {
        let phrases = l.phrases(sl);
        phrases.iter().map(|p| vec_of_words_to_big_int(p.to_vec())).collect::<Vec<_>>()
    }).collect();

    let all_phrase_heads_right: HashSet<i64> = origin_potential_pairings.flat_map(|(_l, r, _sl, sr)| {
        let phrases = r.phrases(sr);
        phrases.iter().map(|p| vec_of_words_to_big_int(p.to_vec())).collect::<Vec<_>>()
    }).collect();

    let all_phrases = phrase_exists_db_filter(conn, all_phrase_heads_left, all_phrase_heads_right)?;

    let mut res = vec![];
    for (lo, ro, lhs, rhs) in all_inputs {
        for answer in attempt_combine_over_with_phrases(&all_phrases, lo, ro, lhs, rhs) {
            res.push(answer);
        }
    }
    Ok(res)
}

pub(crate) fn over_by_hop(
    conn: Option<&PgConnection>,
    phrase: Vec<Word>,
    ortho_by_hop: FailableWordVecToOrthoVec,
    phrase_exists: fn(Option<&PgConnection>, Vec<Word>) -> Result<bool, anyhow::Error>,
) -> Result<Vec<Ortho>, anyhow::Error> {
    let lhs_phrase_head = &phrase[..phrase.len() - 1];
    let rhs_phrase_head = &phrase[1..];

    let orthos_by_hop_left = ortho_by_hop(conn, vec![phrase[0]])?;
    let lhs_by_hop = orthos_by_hop_left
        .iter()
        .filter(|o| o.hop_has_phrase(lhs_phrase_head))
        .filter_map(|o| {
            let axis = o.axis_of_change_between_names_for_hop(phrase[0], phrase[1]);
            if o.axis_length(axis) == phrase.len() - 2 {
                Some((o, axis))
            } else {
                None
            }
        });

    let orthos_by_hop_right = ortho_by_hop(conn, vec![phrase[1]])?;
    let rhs_by_hop = orthos_by_hop_right
        .iter()
        .filter(|o| o.hop_has_phrase(rhs_phrase_head))
        .filter_map(|o| {
            let axis = o.axis_of_change_between_names_for_hop(phrase[1], phrase[2]);
            if o.axis_length(axis) == phrase.len() - 2 {
                Some((o, axis))
            } else {
                None
            }
        });

    let hop_potential_pairings = Itertools::cartesian_product(lhs_by_hop, rhs_by_hop)
        .filter(|((l, _lx), (r, _rx))| l.get_dims() == r.get_dims())
        .map(|((l, lx), (r, rx))| (l, r, lx, rx));

    let all_inputs = hop_potential_pairings;

    let mut res = vec![];
    for (lo, ro, lhs, rhs) in all_inputs {
        for answer in attempt_combine_over(conn, phrase_exists, lo, ro, lhs, rhs)? {
            res.push(answer);
        }
    }
    Ok(res)
}

pub(crate) fn over_by_contents(
    conn: Option<&PgConnection>,
    phrase: Vec<Word>,
    ortho_by_contents: FailableWordVecToOrthoVec,
    phrase_exists: fn(Option<&PgConnection>, Vec<Word>) -> Result<bool, anyhow::Error>,
) -> Result<Vec<Ortho>, anyhow::Error> {
    let lhs_phrase_head = &phrase[..phrase.len() - 1];
    let rhs_phrase_head = &phrase[1..];

    let orthos_by_contents_left = ortho_by_contents(conn, vec![phrase[0]])?;
    let lhs_by_contents = orthos_by_contents_left
        .iter()
        .filter(|o| o.contents_has_phrase(lhs_phrase_head))
        .map(|o| {
            (
                o,
                o.axes_of_change_between_names_for_contents(phrase[0], phrase[1]),
            )
        })
        .flat_map(|(o, axs)| axs.into_iter().map(|axis| (o, axis)).collect::<Vec<_>>())
        .filter(|(o, a)| o.axis_length(*a) == phrase.len() - 2)
        .map(|(o, a)| (o, a));

    let orthos_by_contents_right = ortho_by_contents(conn, vec![phrase[1]])?;
    let rhs_by_contents = orthos_by_contents_right
        .iter()
        .filter(|o| o.contents_has_phrase(rhs_phrase_head))
        .map(|o| {
            (
                o,
                o.axes_of_change_between_names_for_contents(phrase[1], phrase[2]),
            )
        })
        .flat_map(|(o, axs)| axs.into_iter().map(|axis| (o, axis)).collect::<Vec<_>>())
        .filter(|(o, a)| o.axis_length(*a) == phrase.len() - 2)
        .map(|(o, a)| (o, a));

    let contents_potential_pairings =
        Itertools::cartesian_product(lhs_by_contents, rhs_by_contents)
            .filter(|((l, _lx), (r, _rx))| l.get_dims() == r.get_dims())
            .map(|((l, lx), (r, rx))| (l, r, lx, rx));

    let all_inputs = contents_potential_pairings;

    let mut res = vec![];
    for (lo, ro, lhs, rhs) in all_inputs {
        for answer in attempt_combine_over(conn, phrase_exists, lo, ro, lhs, rhs)? {
            res.push(answer);
        }
    }
    Ok(res)
}

pub fn attempt_combine_over(
    conn: Option<&PgConnection>,
    phrase_exists: fn(Option<&PgConnection>, Vec<Word>) -> Result<bool, Error>,
    lo: &Ortho,
    ro: &Ortho,
    left_shift_axis: Word,
    right_shift_axis: Word,
) -> Result<Vec<Ortho>, anyhow::Error> {
    let mut ans = vec![];
    let mut lo_hop_set = lo.get_hop();

    lo_hop_set.remove(&left_shift_axis);
    let lo_hop = Vec::from_iter(lo_hop_set.iter().cloned());

    let mut ro_hop_set = ro.get_hop();
    ro_hop_set.remove(&right_shift_axis);

    let fixed_right_hand: Vec<Word> = ro_hop_set.iter().cloned().collect();

    let lo_hop_len = lo_hop.len();
    let left_hand_coordinate_configurations =
        Itertools::permutations(lo_hop.into_iter(), lo_hop_len);

    for left_mapping in left_hand_coordinate_configurations {
        if axis_lengths_match(&left_mapping, &fixed_right_hand, lo, ro) {
            let mapping = make_mapping(
                left_mapping,
                fixed_right_hand.clone(),
                right_shift_axis,
                left_shift_axis,
            );

            if mapping_works(&mapping, lo, ro, right_shift_axis, left_shift_axis) {
                let ortho_to_add = Ortho::zip_over(lo, ro, &mapping, right_shift_axis);

                if phrases_work(phrase_exists, &ortho_to_add, left_shift_axis, conn)? {
                    ans.push(ortho_to_add);
                }
            }
        }
    }

    Ok(ans)
}

pub fn attempt_combine_over_with_phrases(
    all_phrases: &HashSet<i64>,
    lo: &Ortho,
    ro: &Ortho,
    left_shift_axis: Word,
    right_shift_axis: Word,
) -> Vec<Ortho> {
    let mut ans = vec![];
    let mut lo_hop_set = lo.get_hop();

    lo_hop_set.remove(&left_shift_axis);
    let lo_hop = Vec::from_iter(lo_hop_set.iter().cloned());

    let mut ro_hop_set = ro.get_hop();
    ro_hop_set.remove(&right_shift_axis);

    let fixed_right_hand: Vec<Word> = ro_hop_set.iter().cloned().collect();

    let lo_hop_len = lo_hop.len();
    let left_hand_coordinate_configurations =
        Itertools::permutations(lo_hop.into_iter(), lo_hop_len);

    for left_mapping in left_hand_coordinate_configurations {
        if axis_lengths_match(&left_mapping, &fixed_right_hand, lo, ro) {
            let mapping = make_mapping(
                left_mapping,
                fixed_right_hand.clone(),
                right_shift_axis,
                left_shift_axis,
            );

            if mapping_works(&mapping, lo, ro, right_shift_axis, left_shift_axis) {
                let ortho_to_add = Ortho::zip_over(lo, ro, &mapping, right_shift_axis);

                if phrases_work_precomputed(&all_phrases, &ortho_to_add, left_shift_axis) {
                    ans.push(ortho_to_add);
                }
            }
        }
    }

    ans
}


fn phrases_work(
    phrase_exists: fn(Option<&PgConnection>, Vec<Word>) -> Result<bool, anyhow::Error>,
    ortho_to_add: &Ortho,
    shift_axis: Word,
    conn: Option<&PgConnection>,
) -> Result<bool, anyhow::Error> {
    let phrases = ortho_to_add.phrases(shift_axis);

    for phrase in phrases {
        if !phrase_exists(conn, phrase)? {
            return Ok(false);
        }
    }
    Ok(true)
}

fn phrases_work_precomputed(
    known_phrases: &HashSet<i64>,
    ortho_to_add: &Ortho,
    shift_axis: Word,
) -> bool {
    let phrases = ortho_to_add.phrases(shift_axis);
    let mut desired_phrases = phrases.iter().map(|p| vec_of_words_to_big_int(p.to_vec()));
    desired_phrases.all(|phrase| known_phrases.contains(&phrase))
}

fn axis_lengths_match(left_axes: &[Word], right_axes: &[Word], lo: &Ortho, ro: &Ortho) -> bool {
    let left_lengths: Vec<usize> = left_axes.iter().map(|axis| lo.axis_length(*axis)).collect();
    let right_lengths: Vec<usize> = right_axes
        .iter()
        .map(|axis| ro.axis_length(*axis))
        .collect();

    left_lengths == right_lengths
}

fn mapping_works(
    mapping: &BTreeMap<Word, Word>,
    lo: &Ortho,
    ro: &Ortho,
    origin_shift_axis: Word,
    origin_lhs_known_mapping_member: Word,
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

fn make_mapping(
    left_mapping: Vec<Word>,
    fixed_right_hand: Vec<Word>,
    origin_shift_axis: Word,
    origin_lhs_known_mapping_member: Word,
) -> std::collections::BTreeMap<Word, Word> {
    let mut almost: BTreeMap<Word, Word> = zip(fixed_right_hand, left_mapping).collect();
    almost.insert(origin_shift_axis, origin_lhs_known_mapping_member);
    almost
}

#[cfg(test)]
mod tests {
    use std::collections::HashSet;

    use diesel::PgConnection;
    use maplit::{btreemap, hashset};

    use crate::{
        ortho::Ortho,
        phrase_ortho_handler::{over_by_contents, over_by_hop, over_by_origin},
        Word, vec_of_words_to_big_int,
    };

    use super::axis_lengths_match;

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

    pub(crate) fn fake_phrase_exists_db_filter(
        _conn: Option<&PgConnection>,
        _left: HashSet<i64>,
        _right: HashSet<i64>,
    ) -> Result<HashSet<i64>, anyhow::Error> {
        Ok(hashset! {
            vec_of_words_to_big_int(vec![1, 2, 5]),
            vec_of_words_to_big_int(vec![3, 4, 6]),
        })
    }

    pub(crate) fn fake_phrase_exists_db_filter_two(
        _conn: Option<&PgConnection>,
        _left: HashSet<i64>,
        _right: HashSet<i64>,
    ) -> Result<HashSet<i64>, anyhow::Error> {
        
        Ok(hashset! {
            vec_of_words_to_big_int(vec![1, 2, 5])
        })
    }

    fn fake_phrase_exists_three(
        _conn: Option<&PgConnection>,
        phrase: Vec<Word>,
    ) -> Result<bool, anyhow::Error> {
        let ps = hashset! {
            vec![1, 4, 7],
            vec![2, 5, 8],
            vec![3, 6, 9]
        };
        Ok(ps.contains(&phrase))
    }

    pub(crate) fn fake_phrase_exists_db_filter_three(
        _conn: Option<&PgConnection>,
        _left: HashSet<i64>,
        _right: HashSet<i64>,
    ) -> Result<HashSet<i64>, anyhow::Error> {
        
        Ok(hashset! {
            vec_of_words_to_big_int(vec![1, 4, 7]),
            vec_of_words_to_big_int(vec![2, 5, 8]),
            vec_of_words_to_big_int(vec![3, 6, 9]),
        })
    }

    fn fake_ortho_by_origin(
        _conn: Option<&PgConnection>,
        o: Word,
    ) -> Result<Vec<Ortho>, anyhow::Error> {
        let mut pairs = btreemap! { 1 => vec![Ortho::new(
            1,
            2,
            3,
            4,
        )], 2 => vec![Ortho::new(
            2,
            5,
            4,
            6,
        )]};
        Ok(pairs.entry(o).or_default().to_owned())
    }

    fn fake_ortho_by_origin_two(
        _conn: Option<&PgConnection>,
        o: Word,
    ) -> Result<Vec<Ortho>, anyhow::Error> {
        let small = Ortho::new(2, 4, 5, 25);

        // a b e
        // c d f

        // b e
        // d y
        let l = Ortho::new(1, 2, 3, 4);

        let r = Ortho::new(2, 5, 4, 6);

        let bigger = Ortho::zip_over(
            &l,
            &r,
            &btreemap! {
                5 => 2,
                4 => 3
            },
            5,
        );

        let mut pairs = btreemap! { 1 => vec![bigger], 2 => vec![small]};
        Ok(pairs.entry(o).or_default().to_owned())
    }

    fn fake_ortho_by_origin_three(
        _conn: Option<&PgConnection>,
        o: Word,
    ) -> Result<Vec<Ortho>, anyhow::Error> {
        let l = Ortho::zip_over(
            &Ortho::new(1, 2, 3, 4),
            &Ortho::new(2, 5, 4, 6),
            &btreemap! {
                5 => 2,
                4 => 3
            },
            5,
        );

        // a b e   b e + e g
        // c d f   d f   f h

        let r = Ortho::zip_over(
            &Ortho::new(2, 5, 4, 6),
            &Ortho::new(5, 7, 6, 8),
            &btreemap! {
                7 => 5,
                6 => 4
            },
            7,
        );

        let mut pairs = btreemap! { 1 => vec![l], 2 => vec![r]};
        Ok(pairs.entry(o).or_default().to_owned())
    }

    fn fake_ortho_by_origin_four(
        _conn: Option<&PgConnection>,
        o: Word,
    ) -> Result<Vec<Ortho>, anyhow::Error> {
        // a b e   b e b
        // c d f   d f e
        // h i j   i j g

        // phrase: a b e g
        // LHS: a b e
        // RHS: b e g

        let abcd = Ortho::new(1, 2, 3, 4);

        let fejg = Ortho::new(6, 5, 10, 7);

        let ebfe = Ortho::new(5, 2, 6, 5);

        let bedf = Ortho::new(2, 5, 4, 6);

        let cdhi = Ortho::new(3, 4, 8, 9);

        let dfij = Ortho::new(4, 6, 9, 10);

        // a b  b e
        // c d  d f
        let abecdf = Ortho::zip_over(
            &abcd,
            &bedf,
            &btreemap! {
                5 => 2,
                4 => 3
            },
            5,
        );

        // c d   d f
        // h i   i j
        let cdfhij = Ortho::zip_over(
            &cdhi,
            &dfij,
            &btreemap! {
                6 => 4,
                9 => 8
            },
            6,
        );

        // a b e
        // c d f

        // c d f
        // h i j
        let abecdfhij = Ortho::zip_over(
            &abecdf,
            &cdfhij,
            &btreemap! {
                4 => 2,
                8 => 3
            },
            8,
        );

        // b e  e b
        // d f  f e
        let bebdfe = Ortho::zip_over(
            &bedf,
            &ebfe,
            &btreemap! {
                2 => 5,
                6 => 4
            },
            2,
        );

        // d f   f e
        // i j   j g
        let dfeijg = Ortho::zip_over(
            &dfij,
            &fejg,
            &btreemap! {
                5 => 6,
                10 => 9
            },
            5,
        );

        //  b e b
        //  d f e

        //  d f e
        //  i j g
        let bebdfeijg = Ortho::zip_over(
            &bebdfe,
            &dfeijg,
            &btreemap! {
                6 => 5,
                9 => 4
            },
            9,
        );

        let mut pairs = btreemap! { 1 => vec![abecdfhij], 2 => vec![bebdfeijg]};
        Ok(pairs.entry(o).or_default().to_owned())
    }

    fn fake_ortho_by_contents(
        _conn: Option<&PgConnection>,
        o: Vec<Word>,
    ) -> Result<Vec<Ortho>, anyhow::Error> {
        let mut ans = vec![];

        // a b c
        // d e f

        // d e f
        // g h i

        let abde = Ortho::new(1, 2, 4, 5);

        let bcef = Ortho::new(2, 3, 5, 6);

        let degh = Ortho::new(4, 5, 7, 8);

        let efhi = Ortho::new(5, 6, 8, 9);

        let abcdef = Ortho::zip_over(
            &abde,
            &bcef,
            &btreemap! {
                3 => 2,
                5 => 4
            },
            3,
        );

        let defghi = Ortho::zip_over(
            &degh,
            &efhi,
            &btreemap! {
                6 => 5,
                8 => 7
            },
            6,
        );

        if o.contains(&3) || o.contains(&5) || o.contains(&6) {
            ans.push(abcdef);
        }

        if o.contains(&6) || o.contains(&8) || o.contains(&9) {
            ans.push(defghi);
        }

        Ok(ans)
    }

    fn fake_ortho_by_hop(
        _conn: Option<&PgConnection>,
        o: Vec<Word>,
    ) -> Result<Vec<Ortho>, anyhow::Error> {
        let mut ans = vec![];

        if o.contains(&2) {
            ans.push(Ortho::new(1, 2, 3, 4))
        }

        if o.contains(&3) {
            ans.push(Ortho::new(1, 2, 3, 4))
        }

        if o.contains(&5) {
            ans.push(Ortho::new(2, 5, 4, 6))
        }

        if o.contains(&4) {
            ans.push(Ortho::new(2, 5, 4, 6))
        }

        Ok(ans)
    }

    #[test]
    fn over_by_origin_test() {
        // a b | b e    =   a b e
        // c d | d f        c d f

        let expected = Ortho::zip_over(
            &Ortho::new(1, 2, 3, 4),
            &Ortho::new(2, 5, 4, 6),
            &btreemap! {
                5 => 2,
                4 => 3
            },
            5,
        );

        let actual = over_by_origin(
            None,
            vec![1, 2, 5],
            fake_ortho_by_origin,
            fake_phrase_exists_db_filter,
        )
        .unwrap();

        assert_eq!(vec![expected], actual);
    }

    #[test]
    fn over_filters_mismatched_dims() {
        let actual = over_by_origin(
            None,
            vec![1, 2, 5],
            fake_ortho_by_origin_two,
            fake_phrase_exists_db_filter,
        )
        .unwrap();

        assert_eq!(actual.len(), 0);
    }

    #[test]
    fn over_filters_shift_axis_is_wrong_length() {
        let actual = over_by_origin(
            None,
            vec![1, 2, 5],
            fake_ortho_by_origin_three,
            fake_phrase_exists_db_filter,
        )
        .unwrap();

        assert_eq!(actual.len(), 0);
    }

    #[test]
    fn over_filters_if_the_phrase_wont_result() {
        let actual = over_by_origin(
            None,
            vec![1, 2, 5, 7],
            fake_ortho_by_origin_four,
            fake_phrase_exists_db_filter,
        )
        .unwrap();

        assert_eq!(actual.len(), 0);
    }

    #[test]
    fn axis_lengths_can_match() {
        let fejg = Ortho::new(6, 5, 10, 7);

        let ebfe = Ortho::new(5, 2, 6, 5);

        let bedf = Ortho::new(2, 5, 4, 6);

        let dfij = Ortho::new(4, 6, 9, 10);

        // b e  e b
        // d f  f e
        let bebdfe = Ortho::zip_over(
            &bedf,
            &ebfe,
            &btreemap! {
                2 => 5,
                6 => 4
            },
            2,
        );

        // d f   f e
        // i j   j g
        let dfeijg = Ortho::zip_over(
            &dfij,
            &fejg,
            &btreemap! {
                5 => 6,
                10 => 9
            },
            5,
        );

        // b e b   d f e
        // d f e   i j g
        let yes = axis_lengths_match(&vec![5, 4], &vec![6, 9], &bebdfe.clone(), &dfeijg.clone());

        let no = axis_lengths_match(&vec![4, 5], &vec![6, 9], &bebdfe, &dfeijg);

        assert!(yes);
        assert!(!no);
    }

    #[test]
    fn over_by_origin_filters_if_a_phrase_is_missing_from_db() {
        let actual = over_by_origin(
            None,
            vec![1, 2, 5],
            fake_ortho_by_origin,
            fake_phrase_exists_db_filter_two,
        )
        .unwrap();

        assert_eq!(actual.len(), 0);
    }

    #[test]
    fn over_by_hop_test() {
        // a b | b e    =   a b e
        // c d | d f        c d f
        let expected = Ortho::zip_over(
            &Ortho::new(1, 2, 3, 4),
            &Ortho::new(2, 5, 4, 6),
            &btreemap! {
                5 => 2,
                4 => 3
            },
            5,
        );

        let actual =
            over_by_hop(None, vec![3, 4, 6], fake_ortho_by_hop, fake_phrase_exists).unwrap();

        assert_eq!(vec![expected], actual);
    }

    #[test]
    fn over_by_contents_test() {
        // a b c
        // d e f

        // d e f
        // g h i

        // a b c
        // d e f
        // g h i

        // phrase: c f i

        let abde = Ortho::new(1, 2, 4, 5);

        let bcef = Ortho::new(2, 3, 5, 6);

        let degh = Ortho::new(4, 5, 7, 8);

        let efhi = Ortho::new(5, 6, 8, 9);

        let abcdef = Ortho::zip_over(
            &abde,
            &bcef,
            &btreemap! {
                3 => 2,
                5 => 4
            },
            3,
        );

        let defghi = Ortho::zip_over(
            &degh,
            &efhi,
            &btreemap! {
                6 => 5,
                8 => 7
            },
            6,
        );

        let expected = Ortho::zip_over(
            &abcdef,
            &defghi,
            &btreemap! {
                5 => 2,
                7 => 4
            },
            7,
        );

        let actual = over_by_contents(
            None,
            vec![3, 6, 9],
            fake_ortho_by_contents,
            fake_phrase_exists_three,
        )
        .unwrap();

        assert_eq!(vec![expected], actual);
    }
}
