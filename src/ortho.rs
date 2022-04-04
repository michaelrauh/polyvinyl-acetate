use serde::{Deserialize, Serialize};

use std::collections::{BTreeMap, HashSet};

#[derive(Serialize, Deserialize, Debug, PartialEq)]
pub struct Ortho {
    pub(crate) info: BTreeMap<Location, String>,
}

impl Ortho {
    pub(crate) fn get_origin(&self) -> String {
        let (_k, v) = self
            .info
            .iter()
            .find(|(k, _v)| k.length() == 0)
            .expect("all orthos should have an origin");
        v.to_string()
    }

    pub(crate) fn get_hop(&self) -> HashSet<String> {
        self.info
            .iter()
            .filter_map(|(k, v)| {
                if k.length() == 1 {
                    Some(v.to_string())
                } else {
                    None
                }
            })
            .collect()
    }

    pub(crate) fn get_contents(&self) -> HashSet<String> {
        self.info.iter().map(|(_k, v)| v.to_string()).collect()
    }

    pub(crate) fn new(a: String, b: String, c: String, d: String) -> Ortho {
        println!("creating ortho with {:?}, {:?}, {:?}, {:?}", a, b, c, d);
        let inner_loc_a = BTreeMap::default();
        let mut inner_loc_b = BTreeMap::default();
        let mut inner_loc_c = BTreeMap::default();
        let mut inner_loc_d = BTreeMap::default();
        let mut info = BTreeMap::default();

        inner_loc_b.insert(b.clone(), 1);
        inner_loc_c.insert(c.clone(), 1);
        inner_loc_d.insert(b.clone(), 1);
        inner_loc_d.insert(c.clone(), 1);

        let loc_a = Location { info: inner_loc_a };
        let loc_b = Location { info: inner_loc_b };
        let loc_c = Location { info: inner_loc_c };
        let loc_d = Location { info: inner_loc_d };

        info.insert(loc_a, a);
        info.insert(loc_b, b);
        info.insert(loc_c, c);
        info.insert(loc_d, d);

        Ortho { info }
    }
}

#[derive(Serialize, Deserialize, Ord, PartialOrd, PartialEq, Eq, Debug)]
pub struct Location {
    info: BTreeMap<String, usize>,
}

impl Location {
    fn length(&self) -> usize {
        self.info.iter().fold(0, |acc, (_cur_k, cur_v)| acc + cur_v)
    }
}

#[cfg(test)]
mod tests {
    use std::collections::HashSet;

    use crate::pair_todo_handler::data_vec_to_signed_int;

    use crate::ortho::Ortho;

    #[test]
    fn it_has_an_origin() {
        let example_ortho = Ortho::new(
            "a".to_string(),
            "b".to_string(),
            "c".to_string(),
            "d".to_string(),
        );
        assert_eq!(example_ortho.get_origin(), "a".to_string());
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
        expected.insert("b".to_string());
        expected.insert("c".to_string());
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
        expected.insert("a".to_string());
        expected.insert("b".to_string());
        expected.insert("c".to_string());
        expected.insert("d".to_string());
        assert_eq!(example_ortho.get_contents(), expected);
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
}
