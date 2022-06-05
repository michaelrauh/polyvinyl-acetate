use diesel::PgConnection;

use crate::{ortho::Ortho, FailableStringToOrthoVec, FailableStringVecToOrthoVec};

pub(crate) fn over(
    conn: Option<&PgConnection>,
    phrase: Vec<String>,
    ortho_by_origin: FailableStringToOrthoVec,
    ortho_by_hop: FailableStringVecToOrthoVec,
    ortho_by_contents: FailableStringVecToOrthoVec,
) -> Result<Vec<Ortho>, anyhow::Error> {
    Ok(vec![])
}
