use diesel::PgConnection;

use crate::{ortho::Ortho, FailableStringToOrthoVec, FailableStringVecToOrthoVec};

pub(crate) fn over(
    conn: Option<&PgConnection>,
    phrase: Vec<String>,
    ortho_by_origin: FailableStringToOrthoVec,
    ortho_by_hop: FailableStringVecToOrthoVec,
    ortho_by_contents: FailableStringVecToOrthoVec,
) -> Result<Vec<Ortho>, anyhow::Error> {
    if phrase.len() < 3 {
        return Ok(vec![])
    }
    let lhs_by_origin = ortho_by_origin(conn, &phrase[0]);
    Ok(vec![Ortho::new(
        "a".to_string(),
        "b".to_string(),
        "c".to_string(),
        "d".to_string(),
    )])
}

#[cfg(test)]
mod tests {
    use diesel::PgConnection;
    use maplit::btreemap;

    use crate::ortho::Ortho;

    use super::over;

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

    #[test]
    fn over_with_phrase_of_length_two_or_less_is_empty() {
        let actual = over(
            None,
            vec!["a".to_owned(), "b".to_owned()],
            fake_ortho_by_origin,
            empty_ortho_by_hop,
            empty_ortho_by_contents,
        ).unwrap();

        assert_eq!(actual, vec![])
    }

    #[test]
    fn over_by_origin() {
        // a b | b e
        // c d | d f

        let actual = over(
            None,
            vec!["a".to_owned(), "b".to_owned(), "e".to_owned()],
            fake_ortho_by_origin,
            empty_ortho_by_hop,
            empty_ortho_by_contents,
        );
    }
}
