use crate::ortho::Ortho;
use anyhow::Error;
use diesel::PgConnection;
use itertools::{zip, Itertools};
use std::collections::BTreeMap;
use std::vec::IntoIter;

pub fn up(
    conn: Option<&PgConnection>,
    first_w: &str,
    second_w: &str,
    ortho_by_origin: fn(Option<&PgConnection>, &str) -> Result<Vec<Ortho>, anyhow::Error>,
    ortho_by_hop: fn(Option<&PgConnection>, Vec<String>) -> Result<Vec<Ortho>, anyhow::Error>,
    ortho_by_contents: fn(Option<&PgConnection>, Vec<String>) -> Result<Vec<Ortho>, anyhow::Error>,
    pair_checker: fn(
        Option<&PgConnection>,
        try_left: &str,
        try_right: &str,
    ) -> Result<bool, anyhow::Error>,
) -> Result<Vec<Ortho>, anyhow::Error> {
    let mut ans = vec![];

    for (lo, ro) in get_origin_ortho_pairings(conn, first_w, second_w, ortho_by_origin)?
        .into_iter()
        .chain(get_ortho_pairings(
            conn,
            first_w,
            second_w,
            ortho_by_hop,
            pair_checker,
        )?)
        .chain(get_ortho_pairings(
            conn,
            first_w,
            second_w,
            ortho_by_contents,
            pair_checker,
        )?)
    {
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
                    if mapping_is_complete(
                        conn,
                        pair_checker,
                        mapping.clone(),
                        lo.clone(),
                        ro.clone(),
                    )? && diagonals_do_not_conflict(lo.clone(), ro.clone())
                    {
                        ans.push(Ortho::zip_up(lo.clone(), ro.clone(), mapping));
                    }
                }
            }
        }
    }
    Ok(ans)
}

fn get_origin_ortho_pairings(
    conn: Option<&PgConnection>,
    first_w: &str,
    second_w: &str,
    ortho_by_origin: fn(Option<&PgConnection>, &str) -> Result<Vec<Ortho>, Error>,
) -> Result<Vec<(Ortho, Ortho)>, anyhow::Error> {
    let left_orthos_by_origin: Vec<Ortho> = filter_base(ortho_by_origin(conn, first_w)?);
    let right_orthos_by_origin: Vec<Ortho> = filter_base(ortho_by_origin(conn, second_w)?);

    let potential_pairings_by_origin =
        make_potential_pairings(left_orthos_by_origin, right_orthos_by_origin);
    Ok(potential_pairings_by_origin)
}

fn make_potential_pairings(
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

fn get_ortho_pairings(
    conn: Option<&PgConnection>,
    first_w: &str,
    second_w: &str,
    ortho_by: fn(Option<&PgConnection>, Vec<String>) -> Result<Vec<Ortho>, Error>,
    pair_checker: fn(Option<&PgConnection>, &str, &str) -> Result<bool, Error>,
) -> Result<Vec<(Ortho, Ortho)>, anyhow::Error> {
    let left_orthos_by_contents: Vec<Ortho> =
        filter_base(ortho_by(conn, vec![first_w.to_string()])?);
    let right_orthos_by_contents: Vec<Ortho> =
        filter_base(ortho_by(conn, vec![second_w.to_string()])?);

    let potential_pairings_with_untested_origins: Vec<(Ortho, Ortho)> =
        make_potential_pairings(left_orthos_by_contents, right_orthos_by_contents);

    let mut potential_pairings_by_contents = vec![];
    for (l, r) in potential_pairings_with_untested_origins {
        if pair_checker(conn, &l.get_origin(), &r.get_origin())? {
            potential_pairings_by_contents.push((l, r))
        }
    }
    Ok(potential_pairings_by_contents)
}

fn filter_base(orthos: Vec<Ortho>) -> Vec<Ortho> {
    orthos.into_iter().filter(|o| o.is_base()).collect()
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
    pair_checker: fn(
        Option<&PgConnection>,
        try_left: &str,
        try_right: &str,
    ) -> Result<bool, anyhow::Error>,
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
    pair_checker: fn(
        Option<&PgConnection>,
        try_left: &str,
        try_right: &str,
    ) -> Result<bool, anyhow::Error>,
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

#[cfg(test)]
mod tests {
    use crate::ortho::Ortho;
    use crate::up_handler::up;
    use diesel::PgConnection;
    use maplit::{btreemap, hashset};

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
        o: &str,
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
        o: Vec<String>,
    ) -> Result<Vec<Ortho>, anyhow::Error> {
        Ok(vec![])
    }

    fn empty_ortho_by_contents(
        _conn: Option<&PgConnection>,
        o: Vec<String>,
    ) -> Result<Vec<Ortho>, anyhow::Error> {
        Ok(vec![])
    }

    fn fake_pair_exists(
        _conn: Option<&PgConnection>,
        try_left: &str,
        try_right: &str,
    ) -> Result<bool, anyhow::Error> {
        let pairs = hashset! {("a", "b"), ("c", "d"), ("a", "c"), ("b", "d"), ("e", "f"), ("g", "h"), ("e", "g"), ("f", "h"), ("a", "e"), ("b", "f"), ("c", "g"), ("d", "h")};
        Ok(pairs.contains(&(try_left, try_right)))
    }

    fn fake_pair_exists_three(
        _conn: Option<&PgConnection>,
        try_left: &str,
        try_right: &str,
    ) -> Result<bool, anyhow::Error> {
        let pairs = hashset! {("a", "b"), ("c", "c"), ("a", "c"), ("b", "c"), ("e", "f"), ("c", "h"), ("e", "c"), ("f", "h"), ("a", "e"), ("b", "f"), ("c", "c"), ("c", "h")};
        Ok(pairs.contains(&(try_left, try_right)))
    }

    fn fake_pair_exists_five(
        _conn: Option<&PgConnection>,
        try_left: &str,
        try_right: &str,
    ) -> Result<bool, anyhow::Error> {
        let pairs = hashset! {("a", "e"), ("b", "f"), ("c", "g"), ("d", "h")};
        Ok(pairs.contains(&(try_left, try_right)))
    }

    fn fake_pair_exists_four(
        _conn: Option<&PgConnection>,
        try_left: &str,
        try_right: &str,
    ) -> Result<bool, anyhow::Error> {
        let pairs = hashset! {("a", "b"), ("b", "c"), ("d", "e"), ("e", "f"), ("g", "h"), ("h", "i"), ("j", "k"), ("k", "l"),
        ("a", "d"), ("b", "e"), ("c", "f"), ("g", "j"), ("h", "k"), ("i", "l"), ("a", "g"), ("b", "h"), ("c", "i"), ("d", "j"), ("e", "k"), ("f", "l")};
        Ok(pairs.contains(&(try_left, try_right)))
    }

    fn fake_pair_exists_two(
        _conn: Option<&PgConnection>,
        try_left: &str,
        try_right: &str,
    ) -> Result<bool, anyhow::Error> {
        let pairs = hashset! {("a", "b"), ("c", "d"), ("a", "c"), ("b", "d"), ("e", "f"), ("g", "h"), ("e", "g"), ("f", "h"), ("a", "e"), ("b", "f"), ("c", "g")};
        Ok(pairs.contains(&(try_left, try_right)))
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
            fake_pair_exists,
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
            fake_pair_exists_two,
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
            fake_pair_exists_three,
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
            fake_pair_exists_four,
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
            fake_pair_exists_five,
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
            fake_pair_exists,
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
            fake_pair_exists,
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
