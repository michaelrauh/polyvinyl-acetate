use crate::ortho::Ortho;

use diesel::PgConnection;
type FailableSlicesToOrthoVec =
    fn(Option<&PgConnection>, &str, &str) -> Result<Vec<Ortho>, anyhow::Error>;

pub fn ex_nihilo(
    conn: Option<&PgConnection>,
    first: &str,
    second: &str,
    ffbber: FailableSlicesToOrthoVec,
    fbbfer: FailableSlicesToOrthoVec,
) -> Result<Vec<Ortho>, anyhow::Error> {
    let mut ffbb: Vec<Ortho> = ffbber(conn, first, second)?;
    let mut fbbf: Vec<Ortho> = fbbfer(conn, first, second)?;

    ffbb.append(&mut fbbf);
    Ok(ffbb)
}

#[cfg(test)]
mod tests {
    use diesel::PgConnection;

    use crate::ortho::Ortho;

    use crate::ex_nihilo_handler::ex_nihilo;

    fn fake_fbbf(
        _conn: Option<&PgConnection>,
        _first: &str,
        _second: &str,
    ) -> Result<Vec<Ortho>, anyhow::Error> {
        Ok(vec![Ortho::new(
            "a".to_owned(),
            "b".to_owned(),
            "c".to_owned(),
            "d".to_owned(),
        )])
    }

    fn fake_ffbb(
        _conn: Option<&PgConnection>,
        _first: &str,
        _second: &str,
    ) -> Result<Vec<Ortho>, anyhow::Error> {
        Ok(vec![Ortho::new(
            "e".to_owned(),
            "f".to_owned(),
            "g".to_owned(),
            "h".to_owned(),
        )])
    }

    #[test]
    fn it_stitches_together_given_and_found_and_chains_ffbb_with_fbbf() {
        let actual = ex_nihilo(
            None,
            &"a".to_string(),
            &"b".to_string(),
            fake_ffbb,
            fake_fbbf,
        )
        .unwrap();
        let expected = vec![
            Ortho::new(
                "e".to_owned(),
                "f".to_owned(),
                "g".to_owned(),
                "h".to_owned(),
            ),
            Ortho::new(
                "a".to_string(),
                "b".to_string(),
                "c".to_string(),
                "d".to_string(),
            ),
        ];
        assert_eq!(actual, expected)
    }
}
