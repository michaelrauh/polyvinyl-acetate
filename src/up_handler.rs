use std::collections::HashSet;

use crate::ortho::Ortho;

use crate::{
    up_helper, vec_of_strings_to_signed_int, FailableStringToOrthoVec, FailableStringVecToOrthoVec,
};
use diesel::PgConnection;

pub fn up(
    conn: Option<&PgConnection>,
    first_w: &str,
    second_w: &str,
    ortho_by_origin: FailableStringToOrthoVec,
    ortho_by_hop: FailableStringVecToOrthoVec,
    ortho_by_contents: FailableStringVecToOrthoVec,
    db_filter: fn(
        conn: Option<&PgConnection>,
        first_words: HashSet<String>,
        second_words: HashSet<String>,
    ) -> Result<HashSet<i64>, anyhow::Error>,
) -> Result<Vec<Ortho>, anyhow::Error> {
    let mut ans = vec![];

    let left_orthos_by_origin: Vec<Ortho> = up_helper::filter_base(ortho_by_origin(conn, first_w)?);
    let right_orthos_by_origin: Vec<Ortho> =
        up_helper::filter_base(ortho_by_origin(conn, second_w)?);

    let potential_pairings_by_origin =
        up_helper::make_potential_pairings(left_orthos_by_origin, right_orthos_by_origin);

    let ortho_origin_pairings: Vec<(Ortho, Ortho)> = potential_pairings_by_origin
        .iter()
        .filter(|(lo, ro)| lo.get_dims() == ro.get_dims())
        .cloned()
        .collect();
    let origin_left_vocabulary = ortho_origin_pairings
        .iter()
        .flat_map(|(lo, _ro)| lo.to_vec())
        .map(|(_l, r)| r)
        .collect();
    let origin_right_vocabulary = ortho_origin_pairings
        .iter()
        .flat_map(|(_lo, ro)| ro.to_vec())
        .map(|(_l, r)| r)
        .collect();

    let origin_filtered_pairs =
        db_filter(conn, origin_left_vocabulary, origin_right_vocabulary)?.clone();

    for (lo, ro) in ortho_origin_pairings {
        ans.extend(up_helper::attempt_up(origin_filtered_pairs.clone(), lo, ro));
    }

    let hop_left_orthos: Vec<Ortho> =
        up_helper::filter_base(ortho_by_hop(conn, vec![first_w.to_string()])?);
    let hop_right_orthos: Vec<Ortho> =
        up_helper::filter_base(ortho_by_hop(conn, vec![second_w.to_string()])?);

    let hop_potential_pairings_with_untested_origins: Vec<(Ortho, Ortho)> =
        up_helper::make_potential_pairings(hop_left_orthos, hop_right_orthos)
            .iter()
            .filter(|(lo, ro)| lo.get_dims() == ro.get_dims())
            .cloned()
            .collect();
    let hop_left_vocabulary = hop_potential_pairings_with_untested_origins
        .iter()
        .flat_map(|(lo, _ro)| lo.to_vec())
        .map(|(_l, r)| r)
        .collect();
    let hop_right_vocabulary = hop_potential_pairings_with_untested_origins
        .iter()
        .flat_map(|(_lo, ro)| ro.to_vec())
        .map(|(_l, r)| r)
        .collect();

    let hop_filtered_pairs = db_filter(conn, hop_left_vocabulary, hop_right_vocabulary)?.clone();

    let mut hop_origin_pairings = vec![];
    for (l, r) in hop_potential_pairings_with_untested_origins {
        if hop_filtered_pairs.contains(&vec_of_strings_to_signed_int(vec![
            l.get_origin(),
            r.get_origin(),
        ])) {
            hop_origin_pairings.push((l, r))
        }
    }

    for (lo, ro) in hop_origin_pairings {
        ans.extend(up_helper::attempt_up(hop_filtered_pairs.clone(), lo, ro));
    }

    let contents_left_orthos: Vec<Ortho> =
        up_helper::filter_base(ortho_by_contents(conn, vec![first_w.to_string()])?);
    let contents_right_orthos: Vec<Ortho> =
        up_helper::filter_base(ortho_by_contents(conn, vec![second_w.to_string()])?);

    let contents_potential_pairings_with_untested_origins: Vec<(Ortho, Ortho)> =
        up_helper::make_potential_pairings(contents_left_orthos, contents_right_orthos)
            .iter()
            .filter(|(lo, ro)| lo.get_dims() == ro.get_dims())
            .cloned()
            .collect();
    let contents_left_vocabulary = contents_potential_pairings_with_untested_origins
        .iter()
        .flat_map(|(lo, _ro)| lo.to_vec())
        .map(|(_l, r)| r)
        .collect();
    let contents_right_vocabulary = contents_potential_pairings_with_untested_origins
        .iter()
        .flat_map(|(_lo, ro)| ro.to_vec())
        .map(|(_l, r)| r)
        .collect();

    let contents_filtered_pairs =
        db_filter(conn, contents_left_vocabulary, contents_right_vocabulary)?.clone();

    let mut contents_origin_pairings = vec![];
    for (l, r) in contents_potential_pairings_with_untested_origins {
        if contents_filtered_pairs.contains(&vec_of_strings_to_signed_int(vec![
            l.get_origin(),
            r.get_origin(),
        ])) {
            contents_origin_pairings.push((l, r))
        }
    }

    for (lo, ro) in contents_origin_pairings {
        ans.extend(up_helper::attempt_up(
            contents_filtered_pairs.clone(),
            lo,
            ro,
        ));
    }

    Ok(ans)
}

#[cfg(test)]
mod tests {
    use std::collections::HashSet;

    use crate::up_handler::up;
    use crate::{ortho::Ortho, vec_of_strings_to_signed_int};
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

        let combined = Ortho::zip_up(
            l_one,
            r_one,
            btreemap! { "j".to_string() => "f".to_string(), "k".to_string() => "g".to_string() },
        );

        let mut pairs = btreemap! { "a" => vec![single], "e" => vec![combined]};

        Ok(pairs.entry(o).or_default().to_owned())
    }

    fn fake_ortho_by_origin_three(
        _conn: Option<&PgConnection>,
        o: &str,
    ) -> Result<Vec<Ortho>, anyhow::Error> {
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
        let l = Ortho::zip_over(
            l_one,
            l_two,
            btreemap! { "c".to_string() => "b".to_string(), "e".to_string() => "d".to_string() },
            "c".to_string(),
        );
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
        let r = Ortho::zip_over(
            r_one,
            r_two,
            btreemap! { "i".to_string() => "h".to_string(), "l".to_string() => "j".to_string() },
            "i".to_string(),
        );
        let mut pairs = btreemap! { "a" => vec![l], "g" => vec![r]};

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
        _to_filter: HashSet<String>,
        _second: HashSet<String>,
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
            .map(|(l, r)| vec_of_strings_to_signed_int(vec![l.to_string(), r.to_string()]))
            .collect();
        Ok(res)
    }

    fn fake_pair_hash_db_filter_two(
        _conn: Option<&PgConnection>,
        _to_filter: HashSet<String>,
        _second: HashSet<String>,
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
            .map(|(l, r)| vec_of_strings_to_signed_int(vec![l.to_string(), r.to_string()]))
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
        let expected = Ortho::zip_up(
            Ortho::new(
                "a".to_string(),
                "b".to_string(),
                "c".to_string(),
                "d".to_string(),
            ),
            Ortho::new(
                "e".to_string(),
                "f".to_string(),
                "g".to_string(),
                "h".to_string(),
            ),
            btreemap! {
                "e".to_string() => "a".to_string(),
                "f".to_string() => "b".to_string(),
                "g".to_string() => "c".to_string()
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

        let expected = Ortho::zip_up(
            Ortho::new(
                "a".to_string(),
                "b".to_string(),
                "c".to_string(),
                "d".to_string(),
            ),
            Ortho::new(
                "e".to_string(),
                "f".to_string(),
                "g".to_string(),
                "h".to_string(),
            ),
            btreemap! {
                "e".to_string() => "a".to_string(),
                "f".to_string() => "b".to_string(),
                "g".to_string() => "c".to_string()
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

        let expected = Ortho::zip_up(
            Ortho::new(
                "a".to_string(),
                "b".to_string(),
                "c".to_string(),
                "d".to_string(),
            ),
            Ortho::new(
                "e".to_string(),
                "f".to_string(),
                "g".to_string(),
                "h".to_string(),
            ),
            btreemap! {
                "e".to_string() => "a".to_string(),
                "f".to_string() => "b".to_string(),
                "g".to_string() => "c".to_string()
            },
        );

        assert_eq!(actual, vec![expected]);
    }
}
