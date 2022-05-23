use crate::{ortho::Ortho, up_helper, up_helper::FailableBoolOnPair};
use anyhow::Error;
use diesel::PgConnection;
use std::collections::HashSet;

type FailableStringToOrthoVec =
    fn(Option<&PgConnection>, &str) -> Result<Vec<Ortho>, anyhow::Error>;

pub(crate) fn up(
    conn: Option<&PgConnection>,
    old_ortho: Ortho,
    ortho_by_origin: FailableStringToOrthoVec,
    pair_checker: FailableBoolOnPair,
    forward: fn(Option<&PgConnection>, &str) -> Result<HashSet<String>, Error>,
    backward: fn(Option<&PgConnection>, &str) -> Result<HashSet<String>, Error>,
) -> Result<Vec<Ortho>, anyhow::Error> {
    let mut ans = vec![];
    let projected_forward = forward(conn, &old_ortho.get_origin())?;
    let projected_backward = backward(conn, &old_ortho.get_origin())?;

    let mut orthos_to_right = vec![];
    for f in projected_forward {
        for o in ortho_by_origin(conn, &f)? {
            orthos_to_right.push(o);
        }
    }

    for ro in orthos_to_right {
        up_helper::attempt_up(conn, pair_checker, &mut ans, old_ortho.clone(), ro)?;
    }

    let mut orthos_to_left = vec![];
    for f in projected_backward {
        for o in ortho_by_origin(conn, &f)? {
            orthos_to_left.push(o);
        }
    }
    println!("orthos to left: {:?}", orthos_to_left);
    for lo in orthos_to_left {
        up_helper::attempt_up(conn, pair_checker, &mut ans, lo, old_ortho.clone())?;
    }

    Ok(ans)
}

#[cfg(test)]
mod tests {
    use crate::{ortho::Ortho, up_on_ortho_found_handler::up};
    use diesel::PgConnection;
    use maplit::{btreemap, hashset};
    use std::collections::HashSet;

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

    fn fake_pair_exists(
        _conn: Option<&PgConnection>,
        try_left: &str,
        try_right: &str,
    ) -> Result<bool, anyhow::Error> {
        let pairs = hashset! {("a", "b"), ("c", "d"), ("a", "c"), ("b", "d"), ("e", "f"), ("g", "h"), ("e", "g"), ("f", "h"), ("a", "e"), ("b", "f"), ("c", "g"), ("d", "h")};
        Ok(pairs.contains(&(try_left, try_right)))
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

        let actual = up(
            None,
            left_ortho.clone(),
            fake_ortho_by_origin,
            fake_pair_exists,
            fake_forward,
            fake_backward,
        )
        .unwrap();
        let expected = Ortho::zip_up(
            left_ortho,
            right_ortho,
            btreemap! {
                "e".to_string() => "a".to_string(),
                "f".to_string() => "b".to_string(),
                "g".to_string() => "c".to_string()
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

        let actual = up(
            None,
            right_ortho.clone(),
            fake_ortho_by_origin,
            fake_pair_exists,
            fake_forward,
            fake_backward,
        )
        .unwrap();
        let expected = Ortho::zip_up(
            left_ortho,
            right_ortho,
            btreemap! {
                "e".to_string() => "a".to_string(),
                "f".to_string() => "b".to_string(),
                "g".to_string() => "c".to_string()
            },
        );

        assert_eq!(actual, vec![expected]);
    }
}
