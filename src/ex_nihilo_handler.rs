use crate::{ortho::Ortho, Word};

use diesel::PgConnection;
type FailableSlicesToOrthoVec =
    fn(Option<&PgConnection>, Word, Word) -> Result<Vec<Ortho>, anyhow::Error>;

pub fn ex_nihilo(
    conn: Option<&PgConnection>,
    first: Word,
    second: Word,
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
    use crate::Word;

    use crate::ex_nihilo_handler::ex_nihilo;

    fn fake_fbbf(
        _conn: Option<&PgConnection>,
        _first: Word,
        _second: Word,
    ) -> Result<Vec<Ortho>, anyhow::Error> {
        Ok(vec![Ortho::new(1, 2, 3, 4)])
    }

    fn fake_ffbb(
        _conn: Option<&PgConnection>,
        _first: Word,
        _second: Word,
    ) -> Result<Vec<Ortho>, anyhow::Error> {
        Ok(vec![Ortho::new(5, 6, 7, 8)])
    }

    #[test]
    fn it_stitches_together_given_and_found_and_chains_ffbb_with_fbbf() {
        let actual = ex_nihilo(None, 1, 2, fake_ffbb, fake_fbbf).unwrap();
        let expected = vec![Ortho::new(5, 6, 7, 8), Ortho::new(1, 2, 3, 4)];
        assert_eq!(actual, expected)
    }
}
