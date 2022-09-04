use itertools::Itertools;
use maplit::btreemap;
use serde::{Deserialize, Serialize};

use std::collections::{BTreeMap, HashSet};

use crate::Word;

#[derive(Serialize, Deserialize, Debug, PartialEq, Clone, Eq, PartialOrd, Ord)]
pub struct Ortho {
    pub(crate) info: BTreeMap<Location, Word>,
}

impl Ortho {
    pub(crate) fn get_origin(&self) -> Word {
        let (_k, v) = self
            .info
            .iter()
            .find(|(k, _v)| k.length() == 0)
            .expect("all orthos should have an origin");
        *v
    }

    pub(crate) fn get_hop(&self) -> HashSet<Word> {
        self.info
            .iter()
            .filter_map(|(k, v)| if k.length() == 1 { Some(*v) } else { None })
            .collect()
    }

    pub(crate) fn get_contents(&self) -> HashSet<Word> {
        self.info
            .iter()
            .filter_map(|(k, v)| if k.length() > 1 { Some(*v) } else { None })
            .collect()
    }

    pub(crate) fn is_base(&self) -> bool {
        self.get_bottom_right_corner().each_location_is_length_one()
    }

    pub(crate) fn new(a: Word, b: Word, c: Word, d: Word) -> Ortho {
        Ortho {
            info: btreemap! {Location { info: btreemap! {} } => a, Location {
                info: btreemap! {b => 1},
            } => b, Location {
                info: btreemap! {c => 1},
            } => c, Location {
                info: btreemap! {b => 1, c => 1},
            } => d},
        }
    }

    pub(crate) fn axis_has_phrase(&self, phrase: &[Word], loc: &Location, axis: Word) -> bool {
        phrase
            .iter()
            .skip(1)
            .enumerate()
            .all(|(i, current_phrase_word)| {
                if let Some(name) = self.optional_name_at_location(&loc.add_n(axis, i + 1)) {
                    &name == current_phrase_word
                } else {
                    false
                }
            })
    }

    pub(crate) fn axis_has_exact_phrase(
        &self,
        phrase: &[Word],
        loc: &Location,
        axis: Word,
    ) -> bool {
        self.axis_has_phrase(phrase, loc, axis)
            && self
                .optional_name_at_location(&loc.add_n(axis, phrase.len()))
                .is_none()
    }

    pub(crate) fn origin_has_phrase(&self, phrase: &[Word]) -> bool {
        self.axis_has_phrase(phrase, &Location::default(), phrase[1])
    }

    pub(crate) fn origin_has_full_length_phrase(&self, phrase: &[Word]) -> bool {
        self.axis_has_exact_phrase(phrase, &Location::default(), phrase[1])
    }

    pub(crate) fn hop_has_full_length_phrase(&self, phrase: &[Word]) -> bool {
        let loc = Location::singleton(phrase[0]);

        loc.missing_axes(&self.get_hop())
            .iter()
            .any(|axis| self.axis_has_exact_phrase(phrase, &Location::singleton(phrase[0]), *axis))
    }

    pub(crate) fn contents_has_full_length_phrase(&self, phrase: &[Word]) -> bool {
        let head = &phrase[0];
        let hop = self.get_hop();

        self.location_at_name(*head)
            .into_iter()
            .filter(|loc| loc.is_contents() && loc.is_edge(&hop))
            .any(|starting_location| {
                starting_location
                    .missing_axes(&hop)
                    .into_iter()
                    .any(|axis| self.axis_has_exact_phrase(phrase, starting_location, axis))
            })
    }

    pub(crate) fn get_bottom_right_corner(&self) -> &Location {
        self.info
            .keys()
            .max_by(|left, right| left.length().cmp(&right.length()))
            .expect("don't get the bottom right corner of empty orthos")
    }

    pub(crate) fn get_dims(&self) -> BTreeMap<usize, usize> {
        self.get_bottom_right_corner().dims()
    }

    pub(crate) fn zip_up(
        l: &Ortho,
        r: &Ortho,
        old_axis_to_new_axis: &BTreeMap<Word, Word>,
    ) -> Ortho {
        let shift_axis = r.get_origin();
        let right_with_lefts_coordinate_system: BTreeMap<Location, Word> = r
            .info
            .iter()
            .map(|(k, v)| (k.map_location(old_axis_to_new_axis), *v))
            .collect();
        let shifted_right: BTreeMap<Location, Word> = right_with_lefts_coordinate_system
            .iter()
            .map(|(k, v)| (k.shift_location(shift_axis), *v))
            .collect();
        let combined: BTreeMap<Location, Word> =
            l.info.clone().into_iter().chain(shifted_right).collect();
        Ortho { info: combined }
    }

    pub(crate) fn name_at_location(&self, location: &Location) -> Word {
        *self
            .info
            .get(location)
            .expect("locations must be present to be queried")
    }

    pub(crate) fn optional_name_at_location(&self, location: &Location) -> Option<Word> {
        self.info.get(location).copied()
    }

    pub(crate) fn get_dimensionality(&self) -> usize {
        self.info
            .keys()
            .max_by(|left, right| left.length().cmp(&right.length()))
            .expect("empty orthos are invalid")
            .length()
    }

    pub(crate) fn get_names_at_distance(&self, dist: usize) -> HashSet<Word> {
        self.info
            .iter()
            .filter_map(|(k, v)| if k.length() == dist { Some(*v) } else { None })
            .collect()
    }

    pub(crate) fn zip_over(
        l: &Ortho,
        r: &Ortho,
        mapping: &BTreeMap<Word, Word>,
        shift_axis: Word,
    ) -> Ortho {
        let right_column = r.get_end(shift_axis);
        let shifted: BTreeMap<Location, Word> = right_column
            .into_iter()
            .map(|(k, v)| (k.add(shift_axis), v))
            .collect();
        let mapped = shifted
            .into_iter()
            .map(|(k, v)| (k.map_location(mapping), v));
        let combined: BTreeMap<Location, Word> = l.info.clone().into_iter().chain(mapped).collect();

        Ortho { info: combined }
    }

    pub(crate) fn axis_length(&self, name: Word) -> usize {
        self.info
            .keys()
            .max_by(|left, right| left.count_axis(name).cmp(&right.count_axis(name)))
            .expect("no empty orthos")
            .count_axis(name)
    }

    fn get_end(&self, shift_axis: Word) -> BTreeMap<Location, Word> {
        let axis_length = self.axis_length(shift_axis);
        self.info
            .clone()
            .into_iter()
            .filter(|(k, _v)| k.count_axis(shift_axis) == axis_length)
            .collect()
    }

    fn location_at_name(&self, name: Word) -> Vec<&Location> {
        self.info
            .iter()
            .filter_map(|(loc, n)| if *n == name { Some(loc) } else { None })
            .collect()
    }

    pub(crate) fn to_vec(&self) -> Vec<(&Location, Word)> {
        self.info.iter().map(|(a, b)| (a, *b)).collect()
    }

    pub(crate) fn get_vocabulary(&self) -> impl Iterator<Item = i32> + '_ {
        self.info.iter().map(|(_, b)| *b)
    }

    pub(crate) fn phrases(&self, shift_axis: Word) -> Vec<Vec<Word>> {
        let length = self.axis_length(shift_axis);
        self.info
            .iter()
            .filter(|(loc, _name)| loc.does_not_have_axis(shift_axis))
            .map(|(loc, _name)| self.extract_phrase_along(shift_axis, length, loc))
            .collect()
    }

    fn extract_phrase_along(&self, axis: Word, length: usize, loc: &Location) -> Vec<Word> {
        vec![self.name_at_location(loc)]
            .into_iter()
            .chain((1..length + 1).map(|i| {
                let location = loc.add_n(axis, i);
                self.name_at_location(&location)
            }))
            .collect()
    }

    pub(crate) fn axis_of_change_between_names_for_hop(
        &self,
        from_name: Word,
        to_name: Word,
    ) -> Word {
        let all_locations_for_to_name = self.location_at_name(to_name);
        let to_location = all_locations_for_to_name
            .iter()
            .find(|loc| loc.length() == 2)
            .expect("there should be a name in the hop if it was there before");
        let from_location = Location::default().add(from_name);
        let missing_axes = from_location.missing_axes(&to_location.info.keys().cloned().collect());
        *missing_axes
            .iter()
            .next()
            .expect("there should be an axis of change from hop")
    }

    pub(crate) fn axes_of_change_between_names_for_contents(
        &self,
        from_name: Word,
        to_name: Word,
    ) -> Vec<Word> {
        let from_name_location = self.location_at_name(from_name);
        let from_locations = from_name_location
            .iter()
            .filter(|name| name.is_edge(&self.get_hop()));

        let to_locations = self.location_at_name(to_name);
        let potentials = Itertools::cartesian_product(from_locations, to_locations.iter());
        let valid_potentials = potentials.filter(|(l, r)| (l.length() + 1) == r.length());

        let missing_axeses =
            valid_potentials.map(|(l, r)| r.subtract_adjacent_for_single_axis_name(l));
        missing_axeses.collect()
    }

    pub(crate) fn all_full_length_phrases(&self) -> Vec<Vec<Word>> {
        self.get_hop()
            .iter()
            .flat_map(|axis| {
                let phrases_for_axis = self.phrases(*axis);
                let axis_length = self.axis_length(*axis);
                phrases_for_axis
                    .into_iter()
                    .filter(|phrase| phrase.len() == axis_length + 1)
                    .collect::<Vec<_>>()
            })
            .collect()
    }

    pub(crate) fn origin_phrases(&self) -> Vec<Vec<Word>> {
        self.get_hop()
            .iter()
            .map(|axis| {
                self.extract_phrase_along(*axis, self.axis_length(*axis), &Location::default())
            })
            .collect()
    }
}

#[derive(Serialize, Deserialize, Ord, PartialOrd, PartialEq, Eq, Debug, Clone)]
pub struct Location {
    info: BTreeMap<Word, usize>,
}

impl Location {
    pub fn length(&self) -> usize {
        self.info.iter().fold(0, |acc, (_cur_k, cur_v)| acc + cur_v)
    }

    pub fn map_location(&self, old_axis_to_new_axis: &BTreeMap<Word, Word>) -> Location {
        Location {
            info: self
                .info
                .iter()
                .map(|(k, v)| (*old_axis_to_new_axis.get(k).unwrap_or(k), v.to_owned()))
                .collect(),
        }
    }

    fn shift_location(&self, axis: Word) -> Location {
        let mut other: BTreeMap<Word, usize> = self.info.clone();
        *other.entry(axis).or_insert(0) += 1;
        Location { info: other }
    }

    fn dims(&self) -> BTreeMap<usize, usize> {
        let mut res: BTreeMap<usize, usize> = btreemap! {};
        self.info
            .values()
            .for_each(|v| *res.entry(*v).or_insert(0) += 1);
        res
    }

    pub(crate) fn count_axis(&self, axis: Word) -> usize {
        *self.info.get(&axis).unwrap_or(&0)
    }

    pub(crate) fn add(&self, axis: Word) -> Location {
        self.add_n(axis, 1)
    }

    pub(crate) fn add_n(&self, axis: Word, n: usize) -> Location {
        let mut res: BTreeMap<Word, usize> = self.info.to_owned();
        *res.entry(axis).or_insert(0) += n;
        Location { info: res }
    }

    fn each_location_is_length_one(&self) -> bool {
        self.info.values().all(|v| *v == 1)
    }

    fn is_edge(&self, axes: &HashSet<Word>) -> bool {
        !self.missing_axes(axes).is_empty()
    }

    fn missing_axes(&self, axes: &HashSet<Word>) -> HashSet<Word> {
        axes.iter()
            .filter(|axis| !self.info.keys().contains(axis.to_owned()))
            .cloned()
            .collect()
    }

    fn is_contents(&self) -> bool {
        self.info.values().any(|i| i > &1)
    }

    pub(crate) fn default() -> Location {
        Location { info: btreemap! {} }
    }

    pub(crate) fn singleton(name: Word) -> Location {
        Location {
            info: btreemap! {name => 1},
        }
    }

    fn does_not_have_axis(&self, shift_axis: Word) -> bool {
        !self.info.contains_key(&shift_axis)
    }

    fn subtract_adjacent_for_single_axis_name(&self, other: &Location) -> Word {
        *self
            .info
            .iter()
            .find(|(axis, count)| &other.info.get(axis.to_owned()).unwrap_or(&0) != count)
            .expect("there must be an adjacent name")
            .0
    }
}

#[cfg(test)]
mod tests {
    use std::collections::HashSet;

    use maplit::{btreemap, hashset};

    use crate::pair_todo_handler::data_vec_to_signed_int;

    use crate::ortho::Ortho;

    use super::Location;

    #[test]
    fn location_can_be_added_to() {
        let location = Location {
            info: btreemap! {1 => 1},
        };
        assert_eq!(
            location.add(1),
            Location {
                info: btreemap! {1 => 2}
            }
        );
        assert_eq!(
            location.add(2),
            Location {
                info: btreemap! {1 => 1, 2 => 1}
            }
        );
    }

    #[test]
    fn full_length_phrases_gets_all_of_them() {
        // a b c
        // d e f

        // d e f
        // g h i

        // a b c
        // d e f
        // g h i

        // 1 2 3
        // 4 5 6
        // 7 8 9

        // 1 4 7
        // 2 5 8
        // 3 6 9
        // 1 2 3
        // 4 5 6
        // 7 8 9

        let abde = Ortho::new(1, 2, 4, 5);

        let bcef = Ortho::new(2, 3, 5, 6);

        let degh = Ortho::new(4, 5, 7, 8);

        let efhi = Ortho::new(5, 6, 8, 9);

        let abcdef = Ortho::zip_over(
            &abde,
            &bcef,
            &btreemap! {
                3 => 2,
                5 => 4
            },
            3,
        );

        let defghi = Ortho::zip_over(
            &degh,
            &efhi,
            &btreemap! {
                6 => 5,
                8 => 7
            },
            6,
        );

        let expected = Ortho::zip_over(
            &abcdef,
            &defghi,
            &btreemap! {
                5 => 2,
                7 => 4
            },
            7,
        );

        assert_eq!(
            HashSet::from_iter(expected.all_full_length_phrases()),
            hashset![
                vec![1, 4, 7],
                vec![2, 5, 8],
                vec![3, 6, 9],
                vec![1, 2, 3],
                vec![4, 5, 6],
                vec![7, 8, 9]
            ]
        );
    }

    #[test]
    fn origin_phrases_gets_phrases_that_pass_through_the_origin() {
        // a b c
        // d e f

        // d e f
        // g h i

        // a b c
        // d e f
        // g h i

        // 1 2 3
        // 4 5 6
        // 7 8 9

        // 1 4 7
        // 1 2 3

        let abde = Ortho::new(1, 2, 4, 5);

        let bcef = Ortho::new(2, 3, 5, 6);

        let degh = Ortho::new(4, 5, 7, 8);

        let efhi = Ortho::new(5, 6, 8, 9);

        let abcdef = Ortho::zip_over(
            &abde,
            &bcef,
            &btreemap! {
                3 => 2,
                5 => 4
            },
            3,
        );

        let defghi = Ortho::zip_over(
            &degh,
            &efhi,
            &btreemap! {
                6 => 5,
                8 => 7
            },
            6,
        );

        let expected = Ortho::zip_over(
            &abcdef,
            &defghi,
            &btreemap! {
                5 => 2,
                7 => 4
            },
            7,
        );

        assert_eq!(
            HashSet::from_iter(expected.origin_phrases()),
            hashset![vec![1, 4, 7], vec![1, 2, 3]]
        );
    }

    #[test]
    fn it_has_an_origin() {
        let example_ortho = Ortho::new(1, 2, 3, 4);
        assert_eq!(example_ortho.get_origin(), 1);
    }

    #[test]
    fn it_can_detect_if_it_contains_a_phrase() {
        let example_ortho = Ortho::new(1, 2, 3, 4);

        assert!(example_ortho.origin_has_phrase(&vec![1, 2]));
        assert!(example_ortho.origin_has_phrase(&vec![1, 3]));
        assert!(!example_ortho.origin_has_phrase(&vec![1, 5]));
    }

    #[test]
    fn it_can_detect_if_it_contains_an_exact_full_length_phrase() {
        // 1 2 5
        // 3 4 6

        let wider = Ortho::zip_over(
            &Ortho::new(1, 2, 3, 4),
            &Ortho::new(2, 5, 4, 6),
            &btreemap! {
                5 => 2,
                4 => 3
            },
            5,
        );
        assert!(!wider.origin_has_full_length_phrase(&vec![1, 2]));
        assert!(wider.origin_has_full_length_phrase(&vec![1, 2, 5]));

        assert!(!wider.hop_has_full_length_phrase(&vec![3, 4]));
        assert!(wider.hop_has_full_length_phrase(&vec![3, 4, 6]));

        let abde = Ortho::new(1, 2, 4, 5);

        let bcef = Ortho::new(2, 3, 5, 6);

        let degh = Ortho::new(4, 5, 7, 8);

        let efhi = Ortho::new(5, 6, 8, 9);

        let abcdef = Ortho::zip_over(
            &abde,
            &bcef,
            &btreemap! {
                3 => 2,
                5 => 4
            },
            3,
        );

        let defghi = Ortho::zip_over(
            &degh,
            &efhi,
            &btreemap! {
                6 => 5,
                8 => 7
            },
            6,
        );

        let bigger = Ortho::zip_over(
            &abcdef,
            &defghi,
            &btreemap! {
                5 => 2,
                7 => 4
            },
            7,
        );

        // a b c   1 2 3
        // d e f   4 5 6
        // g h i   7 8 9

        // phrase: c f i   3 6 9
        assert!(!bigger.contents_has_full_length_phrase(&vec![3, 6]));
        assert!(bigger.contents_has_full_length_phrase(&vec![3, 6, 9]));
    }

    #[test]
    fn it_has_a_hop() {
        let example_ortho = Ortho::new(1, 2, 3, 4);
        let mut expected = HashSet::default();
        expected.insert(2);
        expected.insert(3);
        assert_eq!(example_ortho.get_hop(), expected);
    }

    #[test]
    fn it_has_contents() {
        let example_ortho = Ortho::new(1, 2, 3, 4);
        let mut expected = HashSet::default();
        expected.insert(4);
        assert_eq!(example_ortho.get_contents(), expected);

        let tricky_example = Ortho::new(1, 2, 3, 1);
        assert_eq!(tricky_example.get_contents(), hashset! {1});
    }

    #[test]
    fn it_is_rotation_independent() {
        let example_ortho = Ortho::new(1, 2, 3, 4);
        let rotated_ortho = Ortho::new(1, 3, 2, 4);
        assert_eq!(example_ortho, rotated_ortho);
    }

    #[test]
    fn it_hashes_consistently() {
        let example_ortho = Ortho::new(1, 2, 3, 4);
        assert_eq!(
            data_vec_to_signed_int(
                &bincode::serialize(&example_ortho).expect("serialization should work")
            ),
            data_vec_to_signed_int(
                &bincode::serialize(&example_ortho).expect("serialization should work")
            )
        );
    }

    #[test]
    fn it_zips_up() {
        let l = Ortho::new(1, 2, 3, 4);

        let r = Ortho::new(5, 6, 7, 8);

        let mapping = btreemap! {
            5 => 1,
            6 => 2,
            7 => 3
        };

        let actual = Ortho::zip_up(&l, &r, &mapping).info;
        let expected = btreemap! {
            Location { info: btreemap!{} } => 1,
            Location { info: btreemap!{2 => 1} } => 2,
            Location { info: btreemap!{3 => 1} } => 3,
            Location { info: btreemap!{2 => 1, 3 => 1} } => 4,
            Location { info: btreemap!{5 => 1} } => 5,
            Location { info: btreemap!{5 => 1, 2 => 1} } => 6,
            Location { info: btreemap!{5 => 1, 3 => 1} } => 7,
            Location { info: btreemap!{5 => 1, 3 => 1, 2 => 1} } => 8,
        };

        assert_eq!(actual, expected)
    }

    #[test]
    fn it_finds_the_name_at_a_location() {
        let o = Ortho::new(1, 2, 3, 4);

        let actual = o.name_at_location(&Location {
            info: btreemap! {2 => 1, 3 => 1},
        });
        assert_eq!(actual, 4);
    }

    #[test]
    fn it_find_dimensionality() {
        let o = Ortho::new(1, 2, 3, 4);

        assert_eq!(o.get_dimensionality(), 2);
    }

    #[test]
    fn it_gets_all_names_at_a_distance() {
        let o = Ortho::new(1, 2, 3, 4);

        assert_eq!(o.get_names_at_distance(0), hashset! {1});
        assert_eq!(o.get_names_at_distance(1), hashset! {2, 3});
        assert_eq!(o.get_names_at_distance(2), hashset! {4});
    }

    #[test]
    fn it_gets_dims() {
        let o = Ortho::new(1, 2, 3, 4);

        assert_eq!(o.get_dims(), btreemap! {1 => 2});
    }

    #[test]
    fn it_zips_over() {
        let l = Ortho::new(1, 2, 3, 4);

        let r = Ortho::new(2, 5, 4, 6);

        let mapping = btreemap! {
            5 => 2,
            4 => 3
        };

        let shift_axis = 5;

        let actual = Ortho::zip_over(&l, &r, &mapping, shift_axis).info;
        let expected = btreemap! {
            Location { info: btreemap!{} } => 1,
            Location { info: btreemap!{2 => 1} } => 2,
            Location { info: btreemap!{2 => 2} } => 5,
            Location { info: btreemap!{3 => 1} } => 3,
            Location { info: btreemap!{2 => 1, 3 => 1} } => 4,
            Location { info: btreemap!{2 => 2, 3 => 1} } => 6,
        };

        assert_eq!(actual, expected)
    }

    #[test]
    fn it_is_base_dims_unless_it_has_an_axis_of_length_greater_than_one() {
        let l = Ortho::new(1, 2, 3, 4);

        let r = Ortho::new(2, 5, 4, 6);

        let mapping = btreemap! {
            5 => 2,
            4 => 3
        };

        let over_zipped = Ortho::zip_over(&l, &r, &mapping, 5);

        let l_up = Ortho::new(1, 2, 3, 4);

        let r_up = Ortho::new(5, 6, 7, 8);

        let mapping = btreemap! {
            5 => 1,
            6 => 2,
            7 => 3
        };

        let up_zipped = Ortho::zip_up(&l_up, &r_up, &mapping);

        assert_eq!(l.is_base(), true);
        assert_eq!(over_zipped.is_base(), false);
        assert_eq!(up_zipped.is_base(), true);
    }

    #[test]
    fn it_can_detect_missing_axes() {
        let loc = Location {
            info: btreemap! {2 => 5},
        };

        let actual = loc.missing_axes(&hashset! {2, 3});
        let expected = hashset! {3};

        assert_eq!(actual, expected)
    }

    #[test]
    fn it_can_produce_all_phrases_along_an_axis() {
        let l = Ortho::new(1, 2, 3, 4);
        let ans = l.phrases(2);
        assert_eq!(ans, vec![vec![1, 2], vec![3, 4]]);

        let ans_two = l.phrases(3);
        assert_eq!(ans_two, vec![vec![1, 3], vec![2, 4]]);
    }
}
