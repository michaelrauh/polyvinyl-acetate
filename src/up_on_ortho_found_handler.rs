use crate::{ortho::Ortho, up_helper, FailableHashsetWordsToHashsetNumbers, Word};
use anyhow::Error;
use diesel::PgConnection;
use std::collections::HashSet;

pub(crate) fn up_forward(
    conn: Option<&PgConnection>,
    old_ortho: Ortho,
    get_ortho_by_origin_batch: fn(
        Option<&PgConnection>,
        HashSet<Word>,
    ) -> Result<Vec<Ortho>, anyhow::Error>,
    forward: fn(Option<&PgConnection>, Word) -> Result<HashSet<Word>, Error>,
    get_pair_hashes_relevant_to_vocabularies: FailableHashsetWordsToHashsetNumbers,
) -> Result<Vec<Ortho>, anyhow::Error> {
    if !old_ortho.is_base() {
        return Ok(vec![]);
    }

    let mut ans = vec![];

    let projected_forward = forward(conn, old_ortho.get_origin())?;
    let orthos_to_right: Vec<Ortho> = get_ortho_by_origin_batch(conn, projected_forward)?
        .iter()
        .filter(|o| old_ortho.get_dims() == o.get_dims()) // group by dims
        .cloned()
        .collect();

    let forward_left_vocab: HashSet<Word> =
        old_ortho.to_vec().into_iter().map(|(_l, r)| r).collect();
    let forward_right_vocab: HashSet<Word> = orthos_to_right
        .iter()
        .flat_map(|o| o.to_vec())
        .map(|(_l, r)| r)
        .collect();

    let forward_hashes =
        get_pair_hashes_relevant_to_vocabularies(conn, forward_left_vocab, forward_right_vocab)?;

    for ro in orthos_to_right {
        for answer in up_helper::attempt_up(&forward_hashes, &old_ortho, &ro) {
            ans.push(answer);
        }
    }

    Ok(ans)
}

pub(crate) fn up_back(
    conn: Option<&PgConnection>,
    old_ortho: Ortho,
    get_ortho_by_origin_batch: fn(
        Option<&PgConnection>,
        HashSet<Word>,
    ) -> Result<Vec<Ortho>, anyhow::Error>,
    backward: fn(Option<&PgConnection>, Word) -> Result<HashSet<Word>, Error>,
    get_pair_hashes_relevant_to_vocabularies: FailableHashsetWordsToHashsetNumbers,
) -> Result<Vec<Ortho>, anyhow::Error> {
    if !old_ortho.is_base() {
        return Ok(vec![]);
    }

    let mut ans = vec![];

    let projected_backward = backward(conn, old_ortho.get_origin())?;

    let orthos_to_left: Vec<Ortho> = get_ortho_by_origin_batch(conn, projected_backward)?
        .into_iter()
        .filter(|o| old_ortho.get_dims() == o.get_dims()) // group by dims
        .collect();

    let backward_left_vocab = orthos_to_left
        .iter()
        .flat_map(|o| o.to_vec())
        .map(|(_l, r)| r)
        .collect();

    let backward_right_vocab = old_ortho.to_vec().into_iter().map(|(_l, r)| r).collect();
    let backward_hashes =
        get_pair_hashes_relevant_to_vocabularies(conn, backward_left_vocab, backward_right_vocab)?;

    for lo in orthos_to_left {
        for answer in up_helper::attempt_up(&backward_hashes, &lo, &old_ortho) {
            ans.push(answer);
        }
    }

    Ok(ans)
}

#[cfg(test)]
mod tests {
    use crate::{
        ints_to_big_int, ortho::Ortho, up_on_ortho_found_handler::up_back,
        up_on_ortho_found_handler::up_forward, Word,
    };
    use diesel::PgConnection;
    use maplit::{btreemap, hashset};
    use std::collections::HashSet;

    fn fake_forward(
        _conn: Option<&PgConnection>,
        from: Word,
    ) -> Result<HashSet<Word>, anyhow::Error> {
        let mut pairs = btreemap! { 1 => hashset! {12, 2, 3, 5}, 2 => hashset! {4, 11}, 3 => hashset! {4, 5}, 4 => hashset! {11}, 5 => hashset! {11, 12}, 6 => hashset! {13}, 7 => hashset! {13}};
        Ok(pairs.entry(from).or_default().to_owned())
    }

    fn fake_backward(
        _conn: Option<&PgConnection>,
        from: Word,
    ) -> Result<HashSet<Word>, anyhow::Error> {
        let mut pairs = btreemap! { 2 => hashset! {1}, 3 => hashset! {1}, 4 => hashset! {2, 3}, 5 => hashset! {1}, 6 => hashset! {5, 4}, 7 => hashset! {5, 3}, 8 => hashset! {11, 12, 4}};
        Ok(pairs.entry(from).or_default().to_owned())
    }

    fn fake_ortho_by_origin_batch(
        _conn: Option<&PgConnection>,
        _o: HashSet<Word>,
    ) -> Result<Vec<Ortho>, anyhow::Error> {
        let os = vec![Ortho::new(1, 2, 3, 4), Ortho::new(5, 6, 7, 8)];

        Ok(os)
    }

    fn fake_pair_hash_db_filter(
        _conn: Option<&PgConnection>,
        _first_words: HashSet<Word>,
        _second_words: HashSet<Word>,
    ) -> Result<HashSet<i64>, anyhow::Error> {
        let pairs = vec![
            (1, 2),
            (3, 4),
            (1, 3),
            (2, 4),
            (5, 6),
            (7, 8),
            (5, 7),
            (6, 8),
            (1, 5),
            (2, 6),
            (3, 7),
            (4, 8),
        ];
        let res = pairs.iter().map(|(l, r)| ints_to_big_int(*l, *r)).collect();
        Ok(res)
    }

    #[test]
    fn it_creates_up_on_pair_add_when_origin_points_to_origin_from_left() {
        let left_ortho = Ortho::new(1, 2, 3, 4);

        let right_ortho = Ortho::new(5, 6, 7, 8);

        let actual = up_forward(
            None,
            left_ortho.clone(),
            fake_ortho_by_origin_batch,
            fake_forward,
            fake_pair_hash_db_filter,
        )
        .unwrap();
        let expected = Ortho::zip_up(
            &left_ortho,
            &right_ortho,
            &btreemap! {
                5 => 1,
                6 => 2,
                7 => 3
            },
        );

        assert_eq!(actual, vec![expected]);
    }

    #[test]
    fn it_creates_up_on_pair_add_when_origin_points_to_origin_from_right() {
        let left_ortho = Ortho::new(1, 2, 3, 4);

        let right_ortho = Ortho::new(5, 6, 7, 8);

        let actual = up_back(
            None,
            right_ortho.clone(),
            fake_ortho_by_origin_batch,
            fake_backward,
            fake_pair_hash_db_filter,
        )
        .unwrap();
        let expected = Ortho::zip_up(
            &left_ortho,
            &right_ortho,
            &btreemap! {
                5 => 1,
                6 => 2,
                7 => 3
            },
        );

        assert_eq!(actual, vec![expected]);
    }

    #[test]
    fn it_does_not_produce_up_for_non_base_dims_even_if_eligible() {
        let l_one = Ortho::new(1, 2, 4, 5);
        let l_two = Ortho::new(2, 3, 5, 11);
        let l = Ortho::zip_over(&l_one, &l_two, &btreemap! { 3 => 2, 5 => 4 }, 3);

        let actual = up_forward(
            None,
            l,
            fake_ortho_by_origin_batch,
            fake_forward,
            fake_pair_hash_db_filter,
        )
        .unwrap();

        assert_eq!(actual, vec![]);
    }
}
