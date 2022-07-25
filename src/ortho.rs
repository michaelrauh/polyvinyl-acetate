use itertools::Itertools;
use maplit::btreemap;
use serde::{Deserialize, Serialize};

use std::collections::{BTreeMap, HashSet};

#[derive(Serialize, Deserialize, Debug, PartialEq, Clone, Eq, PartialOrd, Ord)]
pub struct Ortho {
    pub(crate) info: BTreeMap<Location, String>,
}

impl Ortho {
    pub(crate) fn get_origin(&self) -> &String {
        let (_k, v) = self
            .info
            .iter()
            .find(|(k, _v)| k.length() == 0)
            .expect("all orthos should have an origin");
        v
    }

    pub(crate) fn get_hop(&self) -> HashSet<&String> {
        self.info
            .iter()
            .filter_map(|(k, v)| if k.length() == 1 { Some(v) } else { None })
            .collect()
    }

    pub(crate) fn get_contents(&self) -> HashSet<&String> {
        self.info
            .iter()
            .filter_map(|(k, v)| if k.length() > 1 { Some(v) } else { None })
            .collect()
    }

    pub(crate) fn is_base(&self) -> bool {
        self.get_bottom_right_corner().each_location_is_length_one()
    }

    pub(crate) fn new(a: String, b: String, c: String, d: String) -> Ortho {
        Ortho {
            info: btreemap! {Location { info: btreemap! {} } => a, Location {
                info: btreemap! {b.clone() => 1},
            } => b.clone(), Location {
                info: btreemap! {c.clone() => 1},
            } => c.clone(), Location {
                info: btreemap! {b.clone() => 1, c.clone() => 1},
            } => d},
        }
    }

    pub(crate) fn contents_has_phrase(&self, phrase: &[String]) -> bool {
        let head = &phrase[0];
        let hop = self.get_hop();

        self.location_at_name(&head)
            .into_iter()
            .filter(|loc| loc.is_contents() && loc.is_edge(&hop))
            .flat_map(|loc| {
                loc.missing_axes(&hop)
                    .into_iter()
                    .find(|axis| self.axis_has_phrase(phrase, &loc, axis))
            })
            .next()
            .is_some()
    }

    pub(crate) fn hop_has_phrase(&self, phrase: &[String]) -> bool {
        let loc = Location::singleton(phrase[0].to_string());

        loc.missing_axes(&self.get_hop())
            .iter()
            .find(|axis| self.axis_has_phrase(phrase, &loc, &axis))
            .is_some()
    }

    pub(crate) fn axis_has_phrase(&self, phrase: &[String], loc: &Location, axis: &String) -> bool {
        for (i, current_phrase_word) in phrase.iter().skip(1).enumerate() {
            let desired = loc.add_n(axis.clone(), i + 1);
            if self
                .optional_name_at_location(desired)
                .unwrap_or(&"".to_string())
                != current_phrase_word
            {
                return false;
            }
        }
        true
    }

    pub(crate) fn origin_has_phrase(&self, phrase: &[String]) -> bool {
        let axis_name = phrase
            .get(1)
            .expect("lists of length greater than one have a second element");

        self.axis_has_phrase(phrase, &Location::default(), axis_name)
    }

    pub(crate) fn get_bottom_right_corner(&self) -> Location {
        let mut max = 0;
        let mut res = Location { info: btreemap! {} };
        for loc in self.info.keys() {
            if loc.length() > max {
                max = loc.length();
                res = loc.clone();
            }
        }
        res
    }

    pub(crate) fn get_dims(&self) -> BTreeMap<usize, usize> {
        self.get_bottom_right_corner().dims()
    }

    pub(crate) fn zip_up(
        l: &Ortho,
        r: &Ortho,
        old_axis_to_new_axis: &BTreeMap<&String, &String>,
    ) -> Ortho {
        let shift_axis = r.get_origin();
        let right_with_lefts_coordinate_system: BTreeMap<Location, String> = r
            .info
            .iter()
            .map(|(k, v)| (k.map_location_lean(old_axis_to_new_axis), v.to_owned()))
            .collect();
        let shifted_right: BTreeMap<Location, String> = right_with_lefts_coordinate_system
            .iter()
            .map(|(k, v)| (k.shift_location(shift_axis.clone()), v.to_owned()))
            .collect();
        let combined: BTreeMap<Location, String> =
            l.info.clone().into_iter().chain(shifted_right).collect();
        Ortho { info: combined }
    }

    pub(crate) fn name_at_location(&self, location: Location) -> String {
        self.info
            .get(&location)
            .expect("locations must be present to be queried")
            .to_string()
    }

    pub(crate) fn optional_name_at_location(&self, location: Location) -> Option<&String> {
        self.info.get(&location)
    }

    pub(crate) fn get_dimensionality(&self) -> usize {
        self.info.keys().fold(0, |acc, cur| {
            if cur.length() > acc {
                cur.length()
            } else {
                acc
            }
        })
    }

    pub(crate) fn get_names_at_distance(&self, dist: usize) -> HashSet<String> {
        self.info
            .iter()
            .filter_map(|(k, v)| {
                if k.length() == dist {
                    Some(v.to_string())
                } else {
                    None
                }
            })
            .collect()
    }

    pub(crate) fn zip_over(
        l: Ortho,
        r: Ortho,
        mapping: BTreeMap<String, String>,
        shift_axis: String,
    ) -> Ortho {
        let right_column = r.get_end(shift_axis.clone());
        let shifted: BTreeMap<Location, String> = right_column
            .iter()
            .map(|(k, v)| (k.add(shift_axis.clone()), v.to_string()))
            .collect();
        let mapped = shifted
            .iter()
            .map(|(k, v)| (k.map_location(&mapping), v.to_string()));
        let combined: BTreeMap<Location, String> = l.info.into_iter().chain(mapped).collect();

        Ortho { info: combined }
    }

    pub(crate) fn axis_length(&self, name: &str) -> usize {
        let mut len = 0;
        for key in self.info.keys() {
            if key.count_axis(name) > len {
                len = key.count_axis(name)
            }
        }
        len
    }

    fn get_end(&self, shift_axis: String) -> BTreeMap<Location, String> {
        let axis_length = self.axis_length(&shift_axis);
        self.info
            .clone()
            .into_iter()
            .filter(|(k, _v)| k.count_axis(&shift_axis) == axis_length)
            .collect()
    }

    fn location_at_name(&self, name: &str) -> Vec<Location> {
        self.info
            .clone()
            .into_iter()
            .filter_map(|(loc, n)| if n == name { Some(loc) } else { None })
            .collect()
    }

    pub(crate) fn to_vec(&self) -> Vec<(Location, String)> {
        self.info
            .iter()
            .map(|(a, b)| (a.clone(), b.clone()))
            .collect()
    }

    pub(crate) fn get_vocabulary(&self) -> impl Iterator<Item = &String> + '_ {
        self.info.iter().map(|(_, b)| b)
    }

    pub(crate) fn phrases(&self, shift_axis: String) -> Vec<Vec<String>> {
        let length = self.axis_length(&shift_axis);
        self.info
            .iter()
            .filter(|(loc, _name)| loc.does_not_have_axis(&shift_axis))
            .map(|(loc, _name)| self.extract_phrase_along(shift_axis.clone(), length, loc))
            .collect()
    }

    fn extract_phrase_along(&self, axis: String, length: usize, loc: &Location) -> Vec<String> {
        let mut res = vec![self.name_at_location(loc.to_owned())];

        for i in 1..length + 1 {
            let location = loc.add_n(axis.clone(), i);
            let name = self.name_at_location(location);
            res.push(name)
        }
        res
    }

    pub(crate) fn axis_of_change_between_names_for_hop(
        &self,
        from_name: String,
        to_name: String,
    ) -> String {
        let all_locations_for_to_name = self.location_at_name(&to_name);
        let to_location = all_locations_for_to_name
            .iter()
            .filter(|loc| loc.length() == 2)
            .collect::<Vec<_>>()[0];
        let from_location = Location::default().add(from_name);
        let missing_axes = from_location.missing_axes(&to_location.info.keys().collect());
        missing_axes
            .iter()
            .next()
            .expect("there should be an axis of change from hop")
            .to_owned()
    }

    pub(crate) fn axes_of_change_between_names_for_contents(
        &self,
        from_name: String,
        to_name: String,
    ) -> Vec<String> {
        let from_name_location = self.location_at_name(&from_name);
        let from_locations = from_name_location
            .iter()
            .filter(|name| name.is_edge(&self.get_hop()));

        let to_locations = self.location_at_name(&to_name);
        let potentials =
            Itertools::cartesian_product(from_locations.cloned(), to_locations.iter().cloned());
        let valid_potentials = potentials
            .filter(|(l, r)| (l.length() + 1) == r.length())
            .collect::<Vec<_>>();

        let missing_axeses = valid_potentials
            .iter()
            .map(|(l, r)| r.subtract_adjacent_for_single_axis_name(l.to_owned()));
        missing_axeses.collect()
    }

    pub(crate) fn all_full_length_phrases(&self) -> Vec<Vec<String>> {
        self.get_hop()
            .iter()
            .flat_map(|axis| {
                let phrases_for_axis = self.phrases(axis.to_owned().to_string());
                let axis_length = self.axis_length(axis);
                phrases_for_axis
                    .iter()
                    .filter(|phrase| phrase.len() == axis_length + 1)
                    .cloned()
                    .collect::<Vec<_>>()
            })
            .collect()
    }
}

#[derive(Serialize, Deserialize, Ord, PartialOrd, PartialEq, Eq, Debug, Clone)]
pub struct Location {
    info: BTreeMap<String, usize>,
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
            info: btreemap! {"a".to_string() => 1},
        };
        assert_eq!(
            location.add("a".to_string()),
            Location {
                info: btreemap! {"a".to_string() => 2}
            }
        );
        assert_eq!(
            location.add("b".to_string()),
            Location {
                info: btreemap! {"a".to_string() => 1, "b".to_string() => 1}
            }
        );
    }

    #[test]
    fn it_has_an_origin() {
        let example_ortho = Ortho::new(
            "a".to_string(),
            "b".to_string(),
            "c".to_string(),
            "d".to_string(),
        );
        assert_eq!(example_ortho.get_origin(), &"a".to_string());
    }

    #[test]
    fn it_can_detect_if_it_contains_a_phrase() {
        let example_ortho = Ortho::new(
            "a".to_string(),
            "b".to_string(),
            "c".to_string(),
            "d".to_string(),
        );

        let wider = Ortho::zip_over(
            Ortho::new(
                "a".to_string(),
                "b".to_string(),
                "c".to_string(),
                "d".to_string(),
            ),
            Ortho::new(
                "b".to_string(),
                "e".to_string(),
                "d".to_string(),
                "f".to_string(),
            ),
            btreemap! {
                "e".to_string() => "b".to_string(),
                "d".to_string() => "c".to_string()
            },
            "e".to_string(),
        );

        let tricky_ortho = Ortho::new(
            "b".to_string(),
            "x".to_string(),
            "b".to_string(),
            "e".to_string(),
        );

        let tricky_two = Ortho::zip_over(
            Ortho::new(
                "b".to_string(),
                "b".to_string(),
                "x".to_string(),
                "x".to_string(),
            ),
            Ortho::new(
                "b".to_string(),
                "b".to_string(),
                "x".to_string(),
                "e".to_string(),
            ),
            btreemap! {
                "b".to_string() => "b".to_string(),
                "x".to_string() => "x".to_string()
            },
            "b".to_string(),
        );

        assert!(example_ortho.origin_has_phrase(&vec!["a".to_string(), "b".to_string()]));
        assert!(example_ortho.hop_has_phrase(&vec!["c".to_string(), "d".to_string()]));
        assert!(example_ortho.origin_has_phrase(&vec!["a".to_string(), "c".to_string()]));
        assert!(example_ortho.hop_has_phrase(&vec!["b".to_string(), "d".to_string()]));
        assert!(!example_ortho.origin_has_phrase(&vec!["a".to_string(), "e".to_string()]));
        assert!(!example_ortho.hop_has_phrase(&vec!["b".to_string(), "a".to_string()]));
        assert!(wider.contents_has_phrase(&vec!["e".to_string(), "f".to_string()]));
        assert!(!wider.contents_has_phrase(&vec!["f".to_string(), "e".to_string()]));
        assert!(tricky_ortho.hop_has_phrase(&vec!["b".to_owned(), "e".to_owned()]));
        assert!(tricky_two.contents_has_phrase(&vec!["b".to_owned(), "e".to_owned()]));
    }

    #[test]
    fn it_has_a_hop() {
        let example_ortho = Ortho::new(
            "a".to_string(),
            "b".to_string(),
            "c".to_string(),
            "d".to_string(),
        );
        let mut expected = HashSet::default();
        let b = "b".to_string();
        let c = "c".to_string();
        expected.insert(&b);
        expected.insert(&c);
        assert_eq!(example_ortho.get_hop(), expected);
    }

    #[test]
    fn it_has_contents() {
        let example_ortho = Ortho::new(
            "a".to_string(),
            "b".to_string(),
            "c".to_string(),
            "d".to_string(),
        );
        let mut expected = HashSet::default();
        let d = "d".to_string();
        expected.insert(&d);
        assert_eq!(example_ortho.get_contents(), expected);

        let tricky_example = Ortho::new(
            "a".to_string(),
            "b".to_string(),
            "c".to_string(),
            "a".to_string(),
        );
        let a = "a".to_string();
        assert_eq!(tricky_example.get_contents(), hashset! {&a});
    }

    #[test]
    fn it_is_rotation_independent() {
        let example_ortho = Ortho::new(
            "a".to_string(),
            "b".to_string(),
            "c".to_string(),
            "d".to_string(),
        );
        let rotated_ortho = Ortho::new(
            "a".to_string(),
            "c".to_string(),
            "b".to_string(),
            "d".to_string(),
        );
        assert_eq!(example_ortho, rotated_ortho);
    }

    #[test]
    fn it_hashes_consistently() {
        let example_ortho = Ortho::new(
            "a".to_string(),
            "b".to_string(),
            "c".to_string(),
            "d".to_string(),
        );
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
        let l = Ortho::new(
            "a".to_string(),
            "b".to_string(),
            "c".to_string(),
            "d".to_string(),
        );

        let r = Ortho::new(
            "e".to_string(),
            "f".to_string(),
            "g".to_string(),
            "h".to_string(),
        );

        let e = &"e".to_string();
        let a = &"a".to_string();
        let f = &"f".to_string();
        let b = &"b".to_string();
        let g = &"g".to_string();
        let c = &"c".to_string();
        let mapping = btreemap! {
            e => a,
            f => b,
            g => c
        };

        let actual = Ortho::zip_up(&l, &r, &mapping).info;
        let expected = btreemap! {
            Location { info: btreemap!{} } => "a".to_string(),
            Location { info: btreemap!{"b".to_string() => 1} } => "b".to_string(),
            Location { info: btreemap!{"c".to_string() => 1} } => "c".to_string(),
            Location { info: btreemap!{"b".to_string() => 1, "c".to_string() => 1} } => "d".to_string(),
            Location { info: btreemap!{"e".to_string() => 1} } => "e".to_string(),
            Location { info: btreemap!{"e".to_string() => 1, "b".to_string() => 1} } => "f".to_string(),
            Location { info: btreemap!{"e".to_string() => 1, "c".to_string() => 1} } => "g".to_string(),
            Location { info: btreemap!{"e".to_string() => 1, "c".to_string() => 1, "b".to_string() => 1} } => "h".to_string(),
        };

        assert_eq!(actual, expected)
    }

    #[test]
    fn it_finds_the_name_at_a_location() {
        let o = Ortho::new(
            "a".to_string(),
            "b".to_string(),
            "c".to_string(),
            "d".to_string(),
        );

        let actual = o.name_at_location(Location {
            info: btreemap! {"b".to_string() => 1, "c".to_string() => 1},
        });
        assert_eq!(actual, "d".to_string());
    }

    #[test]
    fn it_find_dimensionality() {
        let o = Ortho::new(
            "a".to_string(),
            "b".to_string(),
            "c".to_string(),
            "d".to_string(),
        );

        assert_eq!(o.get_dimensionality(), 2);
    }

    #[test]
    fn it_gets_all_names_at_a_distance() {
        let o = Ortho::new(
            "a".to_string(),
            "b".to_string(),
            "c".to_string(),
            "d".to_string(),
        );

        assert_eq!(o.get_names_at_distance(0), hashset! {"a".to_string()});
        assert_eq!(
            o.get_names_at_distance(1),
            hashset! {"b".to_string(), "c".to_string()}
        );
        assert_eq!(o.get_names_at_distance(2), hashset! {"d".to_string()});
    }

    #[test]
    fn it_gets_dims() {
        let o = Ortho::new(
            "a".to_string(),
            "b".to_string(),
            "c".to_string(),
            "d".to_string(),
        );

        assert_eq!(o.get_dims(), btreemap! {1 => 2});
    }

    #[test]
    fn it_zips_over() {
        let l = Ortho::new(
            "a".to_string(),
            "b".to_string(),
            "c".to_string(),
            "d".to_string(),
        );

        let r = Ortho::new(
            "b".to_string(),
            "e".to_string(),
            "d".to_string(),
            "f".to_string(),
        );

        let mapping = btreemap! {
            "e".to_string() => "b".to_string(),
            "d".to_string() => "c".to_string()
        };

        let shift_axis = "e".to_string();

        let actual = Ortho::zip_over(l, r, mapping, shift_axis).info;
        let expected = btreemap! {
            Location { info: btreemap!{} } => "a".to_string(),
            Location { info: btreemap!{"b".to_string() => 1} } => "b".to_string(),
            Location { info: btreemap!{"b".to_string() => 2} } => "e".to_string(),
            Location { info: btreemap!{"c".to_string() => 1} } => "c".to_string(),
            Location { info: btreemap!{"b".to_string() => 1, "c".to_string() => 1} } => "d".to_string(),
            Location { info: btreemap!{"b".to_string() => 2, "c".to_string() => 1} } => "f".to_string(),
        };

        assert_eq!(actual, expected)
    }

    #[test]
    fn it_is_base_dims_unless_it_has_an_axis_of_length_greater_than_one() {
        let l = Ortho::new(
            "a".to_string(),
            "b".to_string(),
            "c".to_string(),
            "d".to_string(),
        );

        let r = Ortho::new(
            "b".to_string(),
            "e".to_string(),
            "d".to_string(),
            "f".to_string(),
        );

        let mapping = btreemap! {
            "e".to_string() => "b".to_string(),
            "d".to_string() => "c".to_string()
        };

        let shift_axis = "e".to_string();

        let over_zipped = Ortho::zip_over(l.clone(), r, mapping, shift_axis);

        let l_up = Ortho::new(
            "a".to_string(),
            "b".to_string(),
            "c".to_string(),
            "d".to_string(),
        );

        let r_up = Ortho::new(
            "e".to_string(),
            "f".to_string(),
            "g".to_string(),
            "h".to_string(),
        );

        let e = &"e".to_string();
        let a = &"a".to_string();
        let f = &"f".to_string();
        let b = &"b".to_string();
        let g = &"g".to_string();
        let c = &"c".to_string();
        let mapping = btreemap! {
            e => a,
            f => b,
            g => c
        };

        let up_zipped = Ortho::zip_up(&l_up, &r_up, &mapping);

        assert_eq!(l.is_base(), true);
        assert_eq!(over_zipped.is_base(), false);
        assert_eq!(up_zipped.is_base(), true);
    }

    #[test]
    fn it_can_detect_missing_axes() {
        let loc = Location {
            info: btreemap! {"b".to_string() => 5},
        };

        let b = "b".to_string();
        let c = "c".to_string();
        let actual = loc.missing_axes(&hashset! {&b, &c});
        let expected = hashset! {"c".to_string()};

        assert_eq!(actual, expected)
    }

    #[test]
    fn it_can_produce_all_phrases_along_an_axis() {
        let l = Ortho::new(
            "a".to_string(),
            "b".to_string(),
            "c".to_string(),
            "d".to_string(),
        );

        let ans = l.phrases("b".to_owned());
        assert_eq!(
            ans,
            vec![
                vec!["a".to_owned(), "b".to_owned()],
                vec!["c".to_owned(), "d".to_owned()]
            ]
        );

        let ans_two = l.phrases("c".to_owned());
        assert_eq!(
            ans_two,
            vec![
                vec!["a".to_owned(), "c".to_owned()],
                vec!["b".to_owned(), "d".to_owned()]
            ]
        );
    }
}

impl Location {
    pub fn length(&self) -> usize {
        self.info.iter().fold(0, |acc, (_cur_k, cur_v)| acc + cur_v)
    }

    pub fn map_location(&self, old_axis_to_new_axis: &BTreeMap<String, String>) -> Location {
        Location {
            info: self
                .info
                .iter()
                .map(|(k, v)| {
                    (
                        old_axis_to_new_axis.get(k).unwrap_or(k).to_owned(),
                        v.to_owned(),
                    )
                })
                .collect(),
        }
    }

    pub fn map_location_lean(&self, old_axis_to_new_axis: &BTreeMap<&String, &String>) -> Location {
        Location {
            info: self
                .info
                .iter()
                .map(|(k, v)| {
                    (
                        old_axis_to_new_axis
                            .get(k)
                            .unwrap_or(&k)
                            .to_owned()
                            .to_owned(),
                        v.to_owned(),
                    )
                })
                .collect(),
        }
    }

    fn shift_location(&self, axis: String) -> Location {
        let mut other: BTreeMap<String, usize> = self.info.clone();
        *other.entry(axis).or_insert(0) += 1;
        Location { info: other }
    }

    fn dims(&self) -> BTreeMap<usize, usize> {
        let mut res: BTreeMap<usize, usize> = btreemap! {};
        for v in self.info.values() {
            *res.entry(*v).or_insert(0) += 1
        }
        res
    }

    pub(crate) fn count_axis(&self, axis: &str) -> usize {
        *self.info.get(axis).unwrap_or(&0)
    }

    pub(crate) fn add(&self, axis: String) -> Location {
        let mut res: BTreeMap<String, usize> = self.info.to_owned();
        *res.entry(axis).or_insert(0) += 1;
        Location { info: res }
    }

    pub(crate) fn add_n(&self, axis: String, n: usize) -> Location {
        let mut res: BTreeMap<String, usize> = self.info.to_owned();
        *res.entry(axis).or_insert(0) += n;
        Location { info: res }
    }

    fn each_location_is_length_one(&self) -> bool {
        self.info.values().all(|v| *v == 1)
    }

    fn is_edge(&self, axes: &HashSet<&String>) -> bool {
        !self.missing_axes(axes).is_empty()
    }

    fn missing_axes(&self, axes: &HashSet<&String>) -> HashSet<String> {
        let kset: HashSet<&String> = self.info.keys().collect();
        axes.difference(&kset).cloned().cloned().collect()
    }

    fn is_contents(&self) -> bool {
        self.info.values().any(|i| i > &1)
    }

    pub(crate) fn default() -> Location {
        Location { info: btreemap! {} }
    }

    pub(crate) fn singleton(name: String) -> Location {
        Location {
            info: btreemap! {name => 1},
        }
    }

    fn does_not_have_axis(&self, shift_axis: &String) -> bool {
        !self.info.contains_key(shift_axis)
    }

    fn subtract_adjacent_for_single_axis_name(&self, other: Location) -> String {
        self.info
            .clone()
            .into_iter()
            .filter(|(axis, count)| other.info.get(axis).unwrap_or(&0) != count)
            .collect::<Vec<_>>()[0]
            .0
            .clone()
    }
}
