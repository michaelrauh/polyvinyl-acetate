use anyhow::Error;
use itertools::{zip, Itertools};
use std::{
    collections::{hash_map::DefaultHasher, BTreeMap, HashSet},
    hash::{Hash, Hasher},
};

use crate::{
    create_todo_entry,
    diesel::query_dsl::filter_dsl::FilterDsl,
    models::{NewOrthotope, NewTodo, Orthotope},
    schema::{
        orthotopes,
        pairs::{dsl::pairs, id},
    },
};
use crate::{
    diesel::{query_dsl::select_dsl::SelectDsl, ExpressionMethods, RunQueryDsl},
    establish_connection,
    models::{Pair, Todo},
    schema,
};
use crate::{
    ortho::Ortho,
    schema::pairs::{first_word, second_word},
};
use diesel::{dsl::exists, BoolExpressionMethods, PgConnection};

pub fn handle_pair_todo(todo: Todo) -> Result<(), anyhow::Error> {
    let conn = establish_connection();
    conn.build_transaction().serializable().run(|| {
        let pair = get_pair(&conn, todo.other)?;
        let new_orthos = new_orthotopes(&conn, pair)?;
        let inserted_orthos = insert_orthotopes(&conn, &new_orthos)?;
        let todos: Vec<NewTodo> = inserted_orthos
            .iter()
            .map(|s| NewTodo {
                domain: "orthotopes".to_owned(),
                other: s.id,
            })
            .collect();
        create_todo_entry(&conn, &todos)?;
        Ok(())
    })
}

fn new_orthotopes(conn: &PgConnection, pair: Pair) -> Result<Vec<NewOrthotope>, anyhow::Error> {
    let ex_nihilo_orthos = ex_nihilo(
        Some(conn),
        &pair.first_word,
        &pair.second_word,
        project_forward,
        project_backward,
    )?;
    let nihilo_iter = ex_nihilo_orthos.iter();
    let up_orthos = up(
        Some(conn),
        &pair.first_word,
        &pair.second_word,
        get_ortho_by_origin,
        pair_exists,
    )?;
    let up_iter = up_orthos.iter();
    let both = nihilo_iter.chain(up_iter);

    let res = both.map(ortho_to_orthotope).collect();
    Ok(res)
}

fn get_ortho_by_origin(conn: Option<&PgConnection>, o: &str) -> Result<Vec<Ortho>, anyhow::Error> {
    use crate::schema::orthotopes::{origin, table as orthotopes};
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

fn up(
    conn: Option<&PgConnection>,
    first_w: &str,
    second_w: &str,
    ortho_by_origin: fn(Option<&PgConnection>, &str) -> Result<Vec<Ortho>, anyhow::Error>,
    pair_checker: fn(
        Option<&PgConnection>,
        try_left: &str,
        try_right: &str,
    ) -> Result<bool, anyhow::Error>,
) -> Result<Vec<Ortho>, anyhow::Error> {
    let left_orthos: Vec<Ortho> = ortho_by_origin(conn, first_w)?
        .into_iter()
        .filter(|o| o.is_base())
        .collect();
    let right_orthos: Vec<Ortho> = ortho_by_origin(conn, second_w)?
        .into_iter()
        .filter(|o| o.is_base())
        .collect();

    let potential_pairings: Vec<(Ortho, Ortho)> =
        Itertools::cartesian_product(left_orthos.iter().cloned(), right_orthos.iter().cloned())
            .collect();
    let mut ans = vec![];
    for (lo, ro) in potential_pairings {
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
                        let new_ortho = Ortho::zip_up(lo.clone(), ro.clone(), mapping);
                        ans.push(new_ortho);
                    }
                }
            }
        }
    }
    Ok(ans)
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

fn pair_exists(
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

fn ortho_to_orthotope(ortho: &Ortho) -> NewOrthotope {
    let information = bincode::serialize(&ortho).expect("serialization should work");
    let origin = ortho.get_origin();
    let hop = Vec::from_iter(ortho.get_hop());
    let contents = Vec::from_iter(ortho.get_contents());
    let info_hash = data_vec_to_signed_int(&information);
    NewOrthotope {
        information,
        origin,
        hop,
        contents,
        info_hash,
    }
}

pub fn data_vec_to_signed_int(x: &[u8]) -> i64 {
    let mut hasher = DefaultHasher::new();
    x.hash(&mut hasher);
    hasher.finish() as i64
}

fn ex_nihilo(
    conn: Option<&PgConnection>,
    first: &str,
    second: &str,
    forward: fn(Option<&PgConnection>, &str) -> Result<HashSet<String>, anyhow::Error>,
    backward: fn(Option<&PgConnection>, &str) -> Result<HashSet<String>, anyhow::Error>,
) -> Result<Vec<Ortho>, anyhow::Error> {
    let mut res = vec![];
    ffbb_search(conn, first, second, forward, backward, &mut res)?;
    fbbf_search(conn, first, second, forward, backward, &mut res)?;
    Ok(res)
}

fn ffbb_search(
    conn: Option<&PgConnection>,
    a: &str,
    b: &str,
    forward: fn(Option<&PgConnection>, &str) -> Result<HashSet<String>, Error>,
    backward: fn(Option<&PgConnection>, &str) -> Result<HashSet<String>, anyhow::Error>,
    res: &mut Vec<Ortho>,
) -> Result<(), anyhow::Error> {
    for d in forward(conn, b)? {
        for c in backward(conn, &d)? {
            if b != c && backward(conn, &c)?.contains(a) {
                res.push(Ortho::new(
                    a.to_string(),
                    b.to_string(),
                    c.clone(),
                    d.clone(),
                ))
            }
        }
    }

    Ok(())
}

fn fbbf_search(
    conn: Option<&PgConnection>,
    b: &str,
    d: &str,
    forward: fn(Option<&PgConnection>, &str) -> Result<HashSet<String>, Error>,
    backward: fn(Option<&PgConnection>, &str) -> Result<HashSet<String>, anyhow::Error>,
    res: &mut Vec<Ortho>,
) -> Result<(), anyhow::Error> {
    for c in backward(conn, d)? {
        if b != c {
            for a in backward(conn, &c)? {
                if forward(conn, &a)?.contains(b) {
                    res.push(Ortho::new(
                        a.clone(),
                        b.to_string(),
                        c.clone(),
                        d.to_string(),
                    ))
                }
            }
        }
    }

    Ok(())
}

fn project_forward(
    conn: Option<&PgConnection>,
    from: &str,
) -> Result<HashSet<String>, anyhow::Error> {
    let seconds_vec: Vec<String> = pairs
        .filter(schema::pairs::first_word.eq(from))
        .select(crate::schema::pairs::second_word)
        .load(conn.expect("do not pass a test dummy in production"))?;

    let seconds = HashSet::from_iter(seconds_vec);
    Ok(seconds)
}

fn project_backward(
    conn: Option<&PgConnection>,
    from: &str,
) -> Result<HashSet<String>, anyhow::Error> {
    let firsts_vec: Vec<String> = pairs
        .filter(schema::pairs::second_word.eq(from))
        .select(crate::schema::pairs::first_word)
        .load(conn.expect("do not pass a test dummy in production"))?;

    let firsts = HashSet::from_iter(firsts_vec);
    Ok(firsts)
}

fn insert_orthotopes(
    conn: &PgConnection,
    new_orthos: &[NewOrthotope],
) -> Result<Vec<Orthotope>, diesel::result::Error> {
    diesel::insert_into(orthotopes::table)
        .values(new_orthos)
        .on_conflict_do_nothing()
        .get_results(conn)
}

fn get_pair(conn: &PgConnection, pk: i32) -> Result<Pair, anyhow::Error> {
    let pair: Pair = pairs
        .filter(id.eq(pk))
        .select(schema::pairs::all_columns)
        .first(conn)?;

    Ok(pair)
}

#[cfg(test)]
mod tests {
    use diesel::PgConnection;
    use maplit::{btreemap, hashset};
    use std::collections::HashSet;

    use crate::ortho::Ortho;

    use super::{ex_nihilo, up};

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

    fn fake_forward(
        _conn: Option<&PgConnection>,
        from: &str,
    ) -> Result<HashSet<String>, anyhow::Error> {
        let mut pairs = btreemap! { "a" => hashset! {"b".to_string(), "c".to_string(), "e".to_string()}, "b" => hashset! {"d".to_string(), "f".to_string()}, "c" => hashset! {"d".to_string(), "e".to_string()}, "d" => hashset! {"f".to_string()}, "e" => hashset! {"f".to_string(), "g".to_string()}, "f" => hashset! {"h".to_string()}, "g" => hashset! {"h".to_string()}};
        Ok(pairs.entry(from).or_default().to_owned())
    }

    fn fake_backward(
        _conn: Option<&PgConnection>,
        from: &str,
    ) -> Result<HashSet<String>, anyhow::Error> {
        let mut pairs = btreemap! { "b" => hashset! {"a".to_string()}, "c" => hashset! {"a".to_string()}, "d" => hashset! {"b".to_string(), "c".to_string()}};
        Ok(pairs.entry(from).or_default().to_owned())
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
    fn it_creates_ex_nihilo_ffbb() {
        let actual = ex_nihilo(
            None,
            &"a".to_string(),
            &"b".to_string(),
            fake_forward,
            fake_backward,
        )
        .unwrap();
        let expected = Ortho::new(
            "a".to_string(),
            "b".to_string(),
            "c".to_string(),
            "d".to_string(),
        );
        assert_eq!(actual, vec![expected])
    }

    #[test]
    fn it_creates_ex_nihilo_fbbf() {
        let actual = ex_nihilo(
            None,
            &"b".to_string(),
            &"d".to_string(),
            fake_forward,
            fake_backward,
        )
        .unwrap();
        let expected = Ortho::new(
            "a".to_string(),
            "b".to_string(),
            "c".to_string(),
            "d".to_string(),
        );

        assert_eq!(actual, vec![expected])
    }

    #[test]
    fn it_creates_up_on_pair_add_when_origin_points_to_origin() {
        let actual = up(None, "a", "e", fake_ortho_by_origin, fake_pair_exists).unwrap();
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
        let actual = up(None, "a", "e", fake_ortho_by_origin, fake_pair_exists_two).unwrap();

        assert_eq!(actual, vec![]);
    }

    #[test]
    fn it_does_not_produce_up_if_that_would_create_a_diagonal_conflict() {
        let actual = up(
            None,
            "a",
            "e",
            fake_ortho_by_origin_two,
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
            fake_pair_exists_five,
        )
        .unwrap();

        assert_eq!(actual, vec![]);
    }

    // stop passing forward to up
    // once origin up is done update the integration test
    // split out up logic into a separate module
}
