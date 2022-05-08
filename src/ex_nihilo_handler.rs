use crate::ortho::Ortho;
use anyhow::Error;
use diesel::PgConnection;
use std::collections::HashSet;

pub fn ex_nihilo(
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

#[cfg(test)]
mod tests {
    use diesel::PgConnection;
    use maplit::{btreemap, hashset};
    use std::collections::HashSet;

    use crate::ortho::Ortho;

    use crate::ex_nihilo_handler::ex_nihilo;

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
}
