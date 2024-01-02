use crate::{ortho::Ortho, up_helper, Holder, Word};

use std::collections::HashSet;

pub(crate) fn up_forward(holder: &mut Holder, old_ortho: Ortho) -> Vec<Ortho> {
    if !old_ortho.is_base() {
        return vec![];
    }
    let mut ans = vec![];

    let projected_forward =
        holder.get_second_words_of_pairs_with_first_word(old_ortho.get_origin());
    let orthos_to_right: Vec<Ortho> = holder
        .get_ortho_with_origin_in(projected_forward)
        .iter()
        .filter(|o| old_ortho.get_dims() == o.get_dims())
        .cloned()
        .collect();

    if orthos_to_right.is_empty() {
        return vec![];
    }

    let forward_left_vocab: HashSet<Word> =
        old_ortho.to_vec().into_iter().map(|(_l, r)| r).collect();
    let forward_right_vocab: HashSet<Word> = orthos_to_right
        .iter()
        .flat_map(|o| o.to_vec())
        .map(|(_l, r)| r)
        .collect();

    let forward_hashes = {
        let firsts: HashSet<i64> =
            holder.get_hashes_of_pairs_with_first_word(Vec::from_iter(forward_left_vocab));
        let seconds: HashSet<i64> =
            holder.get_hashes_of_pairs_with_second_word(Vec::from_iter(forward_right_vocab));

        firsts.intersection(&seconds).cloned().collect()
    };

    for ro in orthos_to_right {
        for answer in up_helper::attempt_up(&forward_hashes, &old_ortho, &ro) {
            ans.push(answer);
        }
    }

    ans
}

pub(crate) fn up_back(holder: &mut Holder, old_ortho: Ortho) -> Vec<Ortho> {
    if !old_ortho.is_base() {
        return vec![];
    }

    let mut ans = vec![];

    let projected_backward = {
        let from = old_ortho.get_origin();
        holder.get_first_words_of_pairs_with_second_word(from)
    };

    let orthos_to_left: Vec<Ortho> = holder
        .get_ortho_with_origin_in(projected_backward)
        .into_iter()
        .filter(|o| old_ortho.get_dims() == o.get_dims())
        .collect();

    if orthos_to_left.is_empty() {
        return vec![];
    }

    let backward_left_vocab: Vec<_> = orthos_to_left
        .iter()
        .flat_map(|o| o.to_vec())
        .map(|(_l, r)| r)
        .collect();

    let backward_right_vocab: Vec<_> = old_ortho.to_vec().into_iter().map(|(_l, r)| r).collect();
    let backward_hashes = {
        let firsts: HashSet<i64> =
            holder.get_hashes_of_pairs_with_first_word(Vec::from_iter(backward_left_vocab));
        let seconds: HashSet<i64> =
            holder.get_hashes_of_pairs_with_second_word(Vec::from_iter(backward_right_vocab));

        firsts.intersection(&seconds).cloned().collect()
    };

    for lo in orthos_to_left {
        for answer in up_helper::attempt_up(&backward_hashes, &lo, &old_ortho) {
            ans.push(answer);
        }
    }

    ans
}

#[cfg(test)]
mod tests {
    use crate::{
        ortho::Ortho, up_on_ortho_found_handler::up_back, up_on_ortho_found_handler::up_forward,
        Holder,
    };
    use maplit::btreemap;

    #[test]
    fn it_creates_up_on_pair_add_when_origin_points_to_origin_from_left() {
        let left_ortho = Ortho::new(1, 2, 3, 4);

        let right_ortho = Ortho::new(5, 6, 7, 8);
        let mut holder: Holder = Holder::new();
        let actual = up_forward(&mut holder, left_ortho.clone());
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
        let mut holder: Holder = Holder::new();
        let actual = up_back(&mut holder, right_ortho.clone());
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
        let mut holder: Holder = Holder::new();
        let actual = up_forward(&mut holder, l);

        assert_eq!(actual, vec![]);
    }
}
