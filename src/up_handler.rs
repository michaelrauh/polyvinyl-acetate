use anyhow::Error;
use itertools::iproduct;
use std::collections::HashSet;

use crate::ortho::Ortho;

use crate::{
    string_refs_to_signed_int, up_helper, FailableHashsetStringsToHashsetNumbers,
    FailableStringToOrthoVec, FailableStringVecToOrthoVec,
};
use diesel::PgConnection;

pub fn up(
    conn: Option<&PgConnection>,
    first_w: &str,
    second_w: &str,
    ortho_by_origin: FailableStringToOrthoVec,
    ortho_by_hop: FailableStringVecToOrthoVec,
    ortho_by_contents: FailableStringVecToOrthoVec,
    db_filter: FailableHashsetStringsToHashsetNumbers,
) -> Result<Vec<Ortho>, anyhow::Error> {
    let left_orthos_by_origin: Vec<Ortho> = up_helper::filter_base(ortho_by_origin(conn, first_w)?);
    let right_orthos_by_origin: Vec<Ortho> =
        up_helper::filter_base(ortho_by_origin(conn, second_w)?);

    let origin_filtered_pairs = get_relevant_pairs(
        db_filter,
        conn,
        &left_orthos_by_origin,
        &right_orthos_by_origin,
    )?;

    let origin_results = iproduct!(left_orthos_by_origin.iter(), right_orthos_by_origin.iter())
        .filter(|(lo, ro)| lo.get_dims() == ro.get_dims())
        .flat_map(|(lo, ro)| up_helper::attempt_up(&origin_filtered_pairs, lo, ro));

    let hop_left_orthos: Vec<Ortho> =
        up_helper::filter_base(ortho_by_hop(conn, vec![first_w.to_string()])?);
    let hop_right_orthos: Vec<Ortho> =
        up_helper::filter_base(ortho_by_hop(conn, vec![second_w.to_string()])?);

    let hop_filtered_pairs =
        get_relevant_pairs(db_filter, conn, &hop_left_orthos, &hop_right_orthos)?;

    let hop_potential_pairings_with_untested_origins =
        iproduct!(hop_left_orthos.iter(), hop_right_orthos.iter())
            .filter(|(lo, ro)| lo.get_dims() == ro.get_dims());

    let hop_origin_pairings = get_valid_pairings(
        hop_potential_pairings_with_untested_origins,
        &hop_filtered_pairs,
    );

    let hop_results =
        hop_origin_pairings.flat_map(|(lo, ro)| up_helper::attempt_up(&hop_filtered_pairs, lo, ro));

    let contents_left_orthos: Vec<Ortho> =
        up_helper::filter_base(ortho_by_contents(conn, vec![first_w.to_string()])?);
    let contents_right_orthos: Vec<Ortho> =
        up_helper::filter_base(ortho_by_contents(conn, vec![second_w.to_string()])?);

    let contents_potential_pairings_with_untested_origins =
        iproduct!(contents_left_orthos.iter(), contents_right_orthos.iter())
            .filter(|(lo, ro)| lo.get_dims() == ro.get_dims());

    let contents_filtered_pairs = get_relevant_pairs(
        db_filter,
        conn,
        &contents_left_orthos,
        &contents_right_orthos,
    )?;

    let contents_origin_pairings = get_valid_pairings(
        contents_potential_pairings_with_untested_origins,
        &contents_filtered_pairs,
    );

    let contents_results = contents_origin_pairings
        .flat_map(|(lo, ro)| up_helper::attempt_up(&contents_filtered_pairs, lo, ro));

    Ok(origin_results
        .chain(hop_results)
        .chain(contents_results)
        .collect())
}

fn get_valid_pairings<'a>(
    hop_potential_pairings_with_untested_origins: impl Iterator<Item = (&'a Ortho, &'a Ortho)> + 'a,
    hop_filtered_pairs: &'a HashSet<i64>,
) -> impl Iterator<Item = (&'a Ortho, &'a Ortho)> + 'a {
    hop_potential_pairings_with_untested_origins.filter_map(|(lo, ro)| {
        if hop_filtered_pairs.contains(&string_refs_to_signed_int(lo.get_origin(), ro.get_origin()))
        {
            Some((lo, ro))
        } else {
            None
        }
    })
}
fn get_relevant_pairs(
    db_filter: FailableHashsetStringsToHashsetNumbers,
    conn: Option<&PgConnection>,
    left_orthos_by_origin: &[Ortho],
    right_orthos_by_origin: &[Ortho],
) -> Result<HashSet<i64>, Error> {
    let origin_filtered_pairs = db_filter(
        conn,
        left_orthos_by_origin
            .iter()
            .flat_map(|lo| lo.get_vocabulary())
            .collect(),
        right_orthos_by_origin
            .iter()
            .flat_map(|ro| ro.get_vocabulary())
            .collect(),
    )?;
    Ok(origin_filtered_pairs)
}

#[cfg(test)]
mod tests {
    use std::collections::HashSet;

    use crate::ortho::Ortho;
    use crate::string_refs_to_signed_int;
    use crate::up_handler::up;
    use diesel::PgConnection;
    use maplit::btreemap;

    fn fake_ortho_by_origin_two(
        _conn: Option<&PgConnection>,
        o: &str,
    ) -> Result<Vec<Ortho>, anyhow::Error> {
        let mut pairs = btreemap! { "a" => vec![Ortho::new(
            "a".to_string(),
            "b".to_string(),
            "c".to_string(),
            "c".to_string(),
        )], "e" => vec![Ortho::new(
            "e".to_string(),
            "f".to_string(),
            "c".to_string(),
            "h".to_string(),
        )]};
        Ok(pairs.entry(o).or_default().to_owned())
    }

    fn empty_ortho_by_origin(
        _conn: Option<&PgConnection>,
        _o: &str,
    ) -> Result<Vec<Ortho>, anyhow::Error> {
        Ok(vec![])
    }

    fn fake_ortho_by_origin_four(
        _conn: Option<&PgConnection>,
        o: &str,
    ) -> Result<Vec<Ortho>, anyhow::Error> {
        let single = Ortho::new(
            "a".to_string(),
            "b".to_string(),
            "c".to_string(),
            "d".to_string(),
        );
        let l_one = Ortho::new(
            "e".to_string(),
            "f".to_string(),
            "g".to_string(),
            "h".to_string(),
        );

        let r_one = Ortho::new(
            "i".to_string(),
            "j".to_string(),
            "k".to_string(),
            "l".to_string(),
        );

        let j = &"j".to_string();
        let f = &"f".to_string();
        let k = &"k".to_string();
        let g = &"g".to_string();

        let combined = Ortho::zip_up(&l_one, &r_one, &btreemap! { j => f, k => g });

        let mut pairs = btreemap! { "a" => vec![single], "e" => vec![combined]};

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

        let i = &"i".to_string();
        let h = &"h".to_string();
        let l = &"l".to_string();
        let j = &"j".to_string();

        let l_one = Ortho::new(
            "a".to_string(),
            "b".to_string(),
            "d".to_string(),
            "e".to_string(),
        );
        let l_two = Ortho::new(
            "b".to_string(),
            "c".to_string(),
            "e".to_string(),
            "f".to_string(),
        );
        let left_ortho = Ortho::zip_over(&l_one, &l_two, &btreemap! { c => b, e => d }, c);
        let r_one = Ortho::new(
            "g".to_string(),
            "h".to_string(),
            "j".to_string(),
            "k".to_string(),
        );
        let r_two = Ortho::new(
            "h".to_string(),
            "i".to_string(),
            "k".to_string(),
            "l".to_string(),
        );
        let r = Ortho::zip_over(&r_one, &r_two, &btreemap! { i => h, l => j }, i);
        let mut pairs = btreemap! { "a" => vec![left_ortho], "g" => vec![r]};

        Ok(pairs.entry(o).or_default().to_owned())
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
        )], "e" => vec![Ortho::new(
            "e".to_string(),
            "f".to_string(),
            "g".to_string(),
            "h".to_string(),
        )]};
        Ok(pairs.entry(o).or_default().to_owned())
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

        if o.contains(&"f".to_string()) {
            ans.push(Ortho::new(
                "e".to_string(),
                "f".to_string(),
                "g".to_string(),
                "h".to_string(),
            ))
        }

        Ok(ans)
    }

    fn fake_ortho_by_contents(
        _conn: Option<&PgConnection>,
        o: Vec<String>,
    ) -> Result<Vec<Ortho>, anyhow::Error> {
        let mut ans = vec![];

        if o.contains(&"d".to_string()) {
            ans.push(Ortho::new(
                "a".to_string(),
                "b".to_string(),
                "c".to_string(),
                "d".to_string(),
            ))
        }

        if o.contains(&"h".to_string()) {
            ans.push(Ortho::new(
                "e".to_string(),
                "f".to_string(),
                "g".to_string(),
                "h".to_string(),
            ))
        }

        Ok(ans)
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

    fn fake_pair_hash_db_filter(
        _conn: Option<&PgConnection>,
        _to_filter: HashSet<&String>,
        _second: HashSet<&String>,
    ) -> Result<HashSet<i64>, anyhow::Error> {
        let pairs = vec![
            ("a", "b"),
            ("c", "d"),
            ("a", "c"),
            ("b", "d"),
            ("e", "f"),
            ("g", "h"),
            ("e", "g"),
            ("f", "h"),
            ("a", "e"),
            ("b", "f"),
            ("c", "g"),
            ("d", "h"),
        ];
        let res = pairs
            .iter()
            .map(|(l, r)| string_refs_to_signed_int(&l.to_string(), &r.to_string()))
            .collect();
        Ok(res)
    }

    fn fake_pair_hash_db_filter_two(
        _conn: Option<&PgConnection>,
        _to_filter: HashSet<&String>,
        _second: HashSet<&String>,
    ) -> Result<HashSet<i64>, anyhow::Error> {
        let pairs = vec![
            ("a", "b"),
            ("c", "d"),
            ("a", "c"),
            ("b", "d"),
            ("e", "f"),
            ("g", "h"),
            ("e", "g"),
            ("f", "h"),
            ("a", "e"),
            ("b", "f"),
            ("c", "g"),
        ];
        let res = pairs
            .iter()
            .map(|(l, r)| string_refs_to_signed_int(&l.to_string(), &r.to_string()))
            .collect();
        Ok(res)
    }

    #[test]
    fn it_creates_up_on_pair_add_when_origin_points_to_origin() {
        let actual = up(
            None,
            "a",
            "e",
            fake_ortho_by_origin,
            empty_ortho_by_hop,
            empty_ortho_by_contents,
            fake_pair_hash_db_filter,
        )
        .unwrap();

        let e = &"e".to_string();
        let a = &"a".to_string();
        let f = &"f".to_string();
        let b = &"b".to_string();
        let g = &"g".to_string();
        let c = &"c".to_string();

        let expected = Ortho::zip_up(
            &Ortho::new(
                "a".to_string(),
                "b".to_string(),
                "c".to_string(),
                "d".to_string(),
            ),
            &Ortho::new(
                "e".to_string(),
                "f".to_string(),
                "g".to_string(),
                "h".to_string(),
            ),
            &btreemap! {
                e => a,
                f => b,
                g => c
            },
        );

        assert_eq!(actual, vec![expected]);
    }

    #[test]
    fn it_does_not_create_up_when_a_forward_is_missing() {
        let actual = up(
            None,
            "a",
            "e",
            fake_ortho_by_origin,
            empty_ortho_by_hop,
            empty_ortho_by_contents,
            fake_pair_hash_db_filter_two,
        )
        .unwrap();

        assert_eq!(actual, vec![]);
    }

    #[test]
    fn it_does_not_produce_up_if_that_would_create_a_diagonal_conflict() {
        let actual = up(
            None,
            "a",
            "e",
            fake_ortho_by_origin_two,
            empty_ortho_by_hop,
            empty_ortho_by_contents,
            fake_pair_hash_db_filter,
        )
        .unwrap();

        assert_eq!(actual, vec![]);
    }

    #[test]
    fn it_does_not_produce_up_for_non_base_dims_even_if_eligible() {
        let actual = up(
            None,
            "a",
            "g",
            fake_ortho_by_origin_three,
            empty_ortho_by_hop,
            empty_ortho_by_contents,
            fake_pair_hash_db_filter,
        )
        .unwrap();

        assert_eq!(actual, vec![]);
    }

    #[test]
    fn it_only_attempts_to_combine_same_dim_orthos() {
        let actual = up(
            None,
            "a",
            "e",
            fake_ortho_by_origin_four,
            empty_ortho_by_hop,
            empty_ortho_by_contents,
            fake_pair_hash_db_filter,
        )
        .unwrap();

        assert_eq!(actual, vec![]);
    }

    #[test]
    fn it_attempts_to_combine_by_hop() {
        // same combine as before, but b -> f is the pair so it must index into hops
        let actual = up(
            None,
            "b", // a b c d + e f g h
            "f",
            empty_ortho_by_origin,
            fake_ortho_by_hop,
            empty_ortho_by_contents,
            fake_pair_hash_db_filter,
        )
        .unwrap();

        let e = &"e".to_string();
        let a = &"a".to_string();
        let f = &"f".to_string();
        let b = &"b".to_string();
        let g = &"g".to_string();
        let c = &"c".to_string();

        let expected = Ortho::zip_up(
            &Ortho::new(
                "a".to_string(),
                "b".to_string(),
                "c".to_string(),
                "d".to_string(),
            ),
            &Ortho::new(
                "e".to_string(),
                "f".to_string(),
                "g".to_string(),
                "h".to_string(),
            ),
            &btreemap! {
                e => a,
                f => b,
                g => c
            },
        );

        assert_eq!(actual, vec![expected]);
    }

    #[test]
    fn it_attempts_to_combine_by_contents() {
        // same combine as before, but d -> h is the pair so it must index into contents
        let actual = up(
            None,
            "d", // a b c d + e f g h
            "h",
            empty_ortho_by_origin,
            empty_ortho_by_hop,
            fake_ortho_by_contents,
            fake_pair_hash_db_filter,
        )
        .unwrap();

        let e = &"e".to_string();
        let a = &"a".to_string();
        let f = &"f".to_string();
        let b = &"b".to_string();
        let g = &"g".to_string();
        let c = &"c".to_string();

        let expected = Ortho::zip_up(
            &Ortho::new(
                "a".to_string(),
                "b".to_string(),
                "c".to_string(),
                "d".to_string(),
            ),
            &Ortho::new(
                "e".to_string(),
                "f".to_string(),
                "g".to_string(),
                "h".to_string(),
            ),
            &btreemap! {
                e => a,
                f => b,
                g => c
            },
        );

        assert_eq!(actual, vec![expected]);
    }
}
