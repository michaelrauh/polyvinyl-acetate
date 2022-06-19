use std::collections::HashSet;

use anyhow::Error;
use diesel::PgConnection;

use crate::{ortho::Ortho, phrase_ortho_handler::attempt_combine_over, FailableStringToOrthoVec};

pub(crate) fn over(
    conn: Option<&diesel::PgConnection>,
    old_orthotope: crate::ortho::Ortho,
    get_ortho_by_origin: FailableStringToOrthoVec,
    phrase_exists: fn(Option<&PgConnection>, Vec<String>) -> Result<bool, anyhow::Error>,
    project_forward: fn(Option<&PgConnection>, &str) -> Result<HashSet<String>, Error>,
    project_backward: fn(Option<&PgConnection>, &str) -> Result<HashSet<String>, Error>,
) -> Result<Vec<crate::ortho::Ortho>, anyhow::Error> {
    // considering old_orthotope to be the left hand side
    let all_phrases = old_orthotope.all_full_length_phrases();
    let mut rhs_phrases = vec![]; // phrase with extension into the next ortho going to the right

    for old_phrase in all_phrases.clone() {
        let last = &old_phrase.last().expect("orthos cannot have empty phrases");
        let nexts = project_forward(conn, last)?;
        for next in nexts {
            let current_phrase = old_phrase
                .iter()
                .chain(vec![next].iter())
                .map(|s| s.to_owned())
                .collect::<Vec<_>>();
            if phrase_exists(conn, current_phrase.clone())? {
                rhs_phrases.push(current_phrase.to_vec());
            }
        }
    }

    let mut forward_potential_pairings: Vec<(Ortho, Ortho, String, String)> = vec![];
    for phrase in rhs_phrases {
        for potential_ortho in get_ortho_by_origin(conn, &phrase[1])? {
            if potential_ortho.origin_has_phrase(&phrase[1..].to_vec())
                && potential_ortho.axis_length(&phrase[2]) == phrase.len() - 2
                && old_orthotope.get_dims() == potential_ortho.get_dims()
            {
                forward_potential_pairings.push((
                    old_orthotope.clone(),
                    potential_ortho,
                    phrase[1].clone(),
                    phrase[2].clone(),
                ));
            }
        }
    }

    // considering old orthotope to be on the right hand side
    let mut lhs_phrases = vec![]; // phrase with extension into the previous ortho going to the left

    for old_phrase in all_phrases {
        let prevs = project_backward(conn, &old_phrase[0])?;
        for prev in prevs {
            let current_phrase = vec![prev]
                .iter()
                .chain(old_phrase.iter())
                .map(|s| s.to_owned())
                .collect::<Vec<_>>();
            if phrase_exists(conn, current_phrase.clone())? {
                lhs_phrases.push(current_phrase.to_vec());
            }
        }
    }

    let mut backward_potential_pairings: Vec<(Ortho, Ortho, String, String)> = vec![];
    for phrase in lhs_phrases {
        for potential_ortho in get_ortho_by_origin(conn, &phrase[0])? {
            if potential_ortho.origin_has_phrase(&phrase[..phrase.len() - 1].to_vec())
                && potential_ortho.axis_length(&phrase[1]) == phrase.len() - 2
                && old_orthotope.get_dims() == potential_ortho.get_dims()
            {
                backward_potential_pairings.push((
                    potential_ortho,
                    old_orthotope.clone(),
                    phrase[1].clone(),
                    phrase[2].clone(),
                ));
            }
        }
    }

    let potential_pairings: Vec<(Ortho, Ortho, String, String)> = backward_potential_pairings
        .iter()
        .chain(forward_potential_pairings.iter())
        .map(|x| x.to_owned())
        .collect();

    let ans = attempt_combine_over(conn, phrase_exists, potential_pairings)?;

    Ok(ans)
}

#[cfg(test)]
mod tests {
    use crate::{ortho::Ortho, over_on_ortho_found_handler::over};
    use diesel::PgConnection;
    use maplit::{btreemap, hashset};
    use std::collections::HashSet;

    fn fake_forward(
        _conn: Option<&PgConnection>,
        from: &str,
    ) -> Result<HashSet<String>, anyhow::Error> {
        // a b  | b e
        // c d  | d f
        let mut pairs = btreemap! { "b" => hashset! {"d".to_string(), "e".to_string()}};
        Ok(pairs.entry(from).or_default().to_owned())
    }

    fn fake_backward(
        _conn: Option<&PgConnection>,
        from: &str,
    ) -> Result<HashSet<String>, anyhow::Error> {
        // a b  | b e
        // c d  | d f
        let mut pairs = btreemap! { "b" => hashset! {"a".to_string()}};
        Ok(pairs.entry(from).or_default().to_owned())
    }

    fn empty_backward(
        _conn: Option<&PgConnection>,
        _from: &str,
    ) -> Result<HashSet<String>, anyhow::Error> {
        let pairs = hashset! {};
        Ok(pairs)
    }

    fn empty_forward(
        _conn: Option<&PgConnection>,
        _from: &str,
    ) -> Result<HashSet<String>, anyhow::Error> {
        let pairs = hashset! {};
        Ok(pairs)
    }

    fn fake_ortho_by_origin(
        _conn: Option<&PgConnection>,
        o: &str,
    ) -> Result<Vec<Ortho>, anyhow::Error> {
        let mut pairs = btreemap! { "b" => vec![Ortho::new(
            "b".to_string(),
            "e".to_string(),
            "d".to_string(),
            "f".to_string(),
        )]};
        Ok(pairs.entry(o).or_default().to_owned())
    }

    fn fake_ortho_by_origin_two(
        _conn: Option<&PgConnection>,
        o: &str,
    ) -> Result<Vec<Ortho>, anyhow::Error> {
        let mut pairs = btreemap! { "a" => vec![Ortho::new(
            "a".to_string(),
            "b".to_string(),
            "c".to_string(),
            "d".to_string(),
        )]};
        Ok(pairs.entry(o).or_default().to_owned())
    }

    fn fake_phrase_exists(
        _conn: Option<&PgConnection>,
        phrase: Vec<String>,
    ) -> Result<bool, anyhow::Error> {
        let ps = hashset! {
            vec!["a".to_owned(), "b".to_owned(), "e".to_owned()],
            vec!["c".to_owned(),"d".to_owned(), "f".to_owned()]
        };
        Ok(ps.contains(&phrase))
    }

    #[test]
    fn it_creates_over_when_origin_points_to_origin_from_left() {
        // a b  | b e
        // c d  | d f

        let left_ortho = Ortho::new(
            "a".to_string(),
            "b".to_string(),
            "c".to_string(),
            "d".to_string(),
        );

        let right_ortho = Ortho::new(
            "b".to_string(),
            "e".to_string(),
            "d".to_string(),
            "f".to_string(),
        );

        let actual = over(
            None,
            left_ortho.clone(),
            fake_ortho_by_origin,
            fake_phrase_exists,
            fake_forward,
            empty_backward,
        )
        .unwrap();

        let expected = Ortho::zip_over(
            left_ortho,
            right_ortho,
            btreemap! {
                "e".to_string() => "b".to_string(),
                "d".to_string() => "c".to_string()
            },
            "e".to_string(),
        );

        assert_eq!(actual, vec![expected]);
    }

    #[test]
    fn it_creates_over_when_origin_points_to_origin_from_right() {
        // a b  | b e
        // c d  | d f

        let left_ortho = Ortho::new(
            "a".to_string(),
            "b".to_string(),
            "c".to_string(),
            "d".to_string(),
        );

        let right_ortho = Ortho::new(
            "b".to_string(),
            "e".to_string(),
            "d".to_string(),
            "f".to_string(),
        );

        let actual = over(
            None,
            right_ortho.clone(),
            fake_ortho_by_origin_two,
            fake_phrase_exists,
            empty_forward,
            fake_backward,
        )
        .unwrap();

        let expected = Ortho::zip_over(
            left_ortho,
            right_ortho,
            btreemap! {
                "e".to_string() => "b".to_string(),
                "d".to_string() => "c".to_string()
            },
            "e".to_string(),
        );

        assert_eq!(actual, vec![expected]);
    }
}
