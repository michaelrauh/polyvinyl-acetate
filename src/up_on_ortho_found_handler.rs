use crate::{ortho::Ortho, up_helper};
use anyhow::Error;
use diesel::PgConnection;
use std::collections::HashSet;

type FailableStringToOrthoVec =
    fn(Option<&PgConnection>, &str) -> Result<Vec<Ortho>, anyhow::Error>;

pub(crate) fn up(
    conn: Option<&PgConnection>,
    old_ortho: Ortho,
    ortho_by_origin: FailableStringToOrthoVec,
    forward: fn(Option<&PgConnection>, &str) -> Result<HashSet<String>, Error>,
    backward: fn(Option<&PgConnection>, &str) -> Result<HashSet<String>, Error>,
    get_pair_hashes_relevant_to_vocabularies: fn(
        conn: Option<&PgConnection>,
        first_words: HashSet<String>,
        second_words: HashSet<String>,
    ) -> Result<HashSet<i64>, anyhow::Error>,
) -> Result<Vec<Ortho>, anyhow::Error> {
    if !old_ortho.is_base() {
        return Ok(vec![]);
    }

    let mut ans = vec![];

    let projected_forward = forward(conn, &old_ortho.get_origin())?;
    let projected_backward = backward(conn, &old_ortho.get_origin())?;

    let mut orthos_to_right = vec![];
    for f in projected_forward {
        for o in ortho_by_origin(conn, &f)? {
            if old_ortho.get_dims() == o.get_dims() {
                orthos_to_right.push(o);
            }
        }
    }

    let forward_left_vocab: HashSet<String> = old_ortho
        .to_vec()
        .iter()
        .map(|(_l, r)| r)
        .cloned()
        .collect();
    let forward_right_vocab = orthos_to_right
        .iter()
        .flat_map(|o| o.to_vec())
        .map(|(_l, r)| r)
        .collect();

    let forward_hashes = get_pair_hashes_relevant_to_vocabularies(
        conn,
        forward_left_vocab.clone(),
        forward_right_vocab,
    )?;

    for ro in orthos_to_right {
        ans.extend(up_helper::attempt_up(&forward_hashes, &old_ortho, &ro));
    }

    let mut orthos_to_left = vec![];
    for f in projected_backward {
        for o in ortho_by_origin(conn, &f)? {
            if old_ortho.get_dims() == o.get_dims() {
                orthos_to_left.push(o);
            }
        }
    }

    let backward_left_vocab = orthos_to_left
        .iter()
        .flat_map(|o| o.to_vec())
        .map(|(_l, r)| r)
        .collect();
    let backward_right_vocab = forward_left_vocab;
    let backward_hashes =
        get_pair_hashes_relevant_to_vocabularies(conn, backward_left_vocab, backward_right_vocab)?;

    for lo in orthos_to_left {
        ans.extend(up_helper::attempt_up(&backward_hashes, &lo, &old_ortho));
    }

    Ok(ans)
}

#[cfg(test)]
mod tests {
    use crate::{ortho::Ortho, string_refs_to_signed_int, up_on_ortho_found_handler::up};
    use diesel::PgConnection;
    use maplit::{btreemap, hashset};
    use std::collections::HashSet;

    fn fake_forward(
        _conn: Option<&PgConnection>,
        from: &str,
    ) -> Result<HashSet<String>, anyhow::Error> {
        let mut pairs = btreemap! { "a" => hashset! {"g".to_string(), "b".to_string(), "c".to_string(), "e".to_string()}, "b" => hashset! {"d".to_string(), "f".to_string()}, "c" => hashset! {"d".to_string(), "e".to_string()}, "d" => hashset! {"f".to_string()}, "e" => hashset! {"f".to_string(), "g".to_string()}, "f" => hashset! {"h".to_string()}, "g" => hashset! {"h".to_string()}};
        Ok(pairs.entry(from).or_default().to_owned())
    }

    fn fake_backward(
        _conn: Option<&PgConnection>,
        from: &str,
    ) -> Result<HashSet<String>, anyhow::Error> {
        let mut pairs = btreemap! { "b" => hashset! {"a".to_string()}, "c" => hashset! {"a".to_string()}, "d" => hashset! {"b".to_string(), "c".to_string()}, "e" => hashset! {"a".to_string()}, "f" => hashset! {"e".to_string(), "d".to_string()}, "g" => hashset! {"e".to_string(), "c".to_string()}, "h" => hashset! {"f".to_string(), "g".to_string(), "d".to_string()}};
        Ok(pairs.entry(from).or_default().to_owned())
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

    fn fake_pair_hash_db_filter(
        _conn: Option<&PgConnection>,
        _first_words: HashSet<String>,
        _second_words: HashSet<String>,
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

    #[test]
    fn it_creates_up_on_pair_add_when_origin_points_to_origin_from_left() {
        let left_ortho = Ortho::new(
            "a".to_string(),
            "b".to_string(),
            "c".to_string(),
            "d".to_string(),
        );

        let right_ortho = Ortho::new(
            "e".to_string(),
            "f".to_string(),
            "g".to_string(),
            "h".to_string(),
        );

        let e = &"e".to_string();
        let a = &"a".to_string();
        let f = &"f".to_string();
        let b = &"b".to_string();
        let g = &"g".to_string();
        let c = &"c".to_string();

        let actual = up(
            None,
            left_ortho.clone(),
            fake_ortho_by_origin,
            fake_forward,
            fake_backward,
            fake_pair_hash_db_filter,
        )
        .unwrap();
        let expected = Ortho::zip_up(
            &left_ortho,
            &right_ortho,
            &btreemap! {
                e => a,
                f => b,
                g => c
            },
        );

        assert_eq!(actual, vec![expected]);
    }

    #[test]
    fn it_creates_up_on_pair_add_when_origin_points_to_origin_from_right() {
        let left_ortho = Ortho::new(
            "a".to_string(),
            "b".to_string(),
            "c".to_string(),
            "d".to_string(),
        );

        let right_ortho = Ortho::new(
            "e".to_string(),
            "f".to_string(),
            "g".to_string(),
            "h".to_string(),
        );

        let e = &"e".to_string();
        let a = &"a".to_string();
        let f = &"f".to_string();
        let b = &"b".to_string();
        let g = &"g".to_string();
        let c = &"c".to_string();

        let actual = up(
            None,
            right_ortho.clone(),
            fake_ortho_by_origin,
            fake_forward,
            fake_backward,
            fake_pair_hash_db_filter,
        )
        .unwrap();
        let expected = Ortho::zip_up(
            &left_ortho,
            &right_ortho,
            &btreemap! {
                e => a,
                f => b,
                g => c
            },
        );

        assert_eq!(actual, vec![expected]);
    }

    #[test]
    fn it_does_not_produce_up_for_non_base_dims_even_if_eligible() {
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

        let actual = up(
            None,
            l,
            fake_ortho_by_origin_three,
            fake_forward,
            fake_backward,
            fake_pair_hash_db_filter,
        )
        .unwrap();

        assert_eq!(actual, vec![]);
    }
}
