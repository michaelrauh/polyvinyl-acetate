use crate::{
    models::{ExNihilo, Pair},
    ortho::Ortho,
};
use anyhow::Error;
use core::fmt;
use diesel::{sql_query, PgConnection, RunQueryDsl};
use std::collections::{BTreeSet, HashSet};

pub fn ex_nihilo(
    conn: Option<&PgConnection>,
    first: &str,
    second: &str,
    forward: fn(Option<&PgConnection>, &str) -> Result<HashSet<String>, anyhow::Error>,
    backward: fn(Option<&PgConnection>, &str) -> Result<HashSet<String>, anyhow::Error>,
) -> Result<Vec<Ortho>, anyhow::Error> {
    let mut res = vec![];
    ffbb_search(conn, first, second, forward, backward, &mut res)?;

    let single_ffbb: Vec<Ortho> = single_ffbb(conn, first, second)?;
    assert_eq!(res.len(), single_ffbb.len());
    let left: BTreeSet<_> = res.iter().collect();
    let right: BTreeSet<_> = single_ffbb.iter().collect();
    assert_eq!(left, right);

    fbbf_search(conn, first, second, forward, backward, &mut res)?;

    Ok(res)
}

#[flame]
fn single_ffbb(
    conn: Option<&PgConnection>,
    first: &str,
    second: &str,
) -> Result<Vec<Ortho>, anyhow::Error> {
    let query = format!(
        "SELECT CD.first_word, CD.second_word
        FROM pairs CD
        INNER JOIN pairs AC ON AC.second_word=CD.first_word
        INNER JOIN pairs BD ON BD.second_word=CD.second_word AND BD.first_word<>AC.second_word
        WHERE BD.first_word='{}'
        AND AC.first_word='{}';",
        second, first
    );
    let ffbbs: Vec<ExNihilo> =
        sql_query(query).load(conn.expect("do not pass a test dummy in production"))?;

    let res = ffbbs
        .iter()
        .map(|r| {
            Ortho::new(
                first.to_owned(),
                second.to_owned(),
                r.first_word.to_owned(),
                r.second_word.to_owned(),
            )
        })
        .collect();

    Ok(res)
}

#[flame]
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
