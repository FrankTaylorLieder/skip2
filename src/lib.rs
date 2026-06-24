#![allow(dead_code)]

use std::cell::RefCell;
use std::rc::Rc;

type Shared<T> = Rc<RefCell<T>>;

struct Node {
    key: Option<usize>,
    value: String,
    nexts: Vec<NodeRef>,
}

type NodeRef = Option<Shared<Node>>;

const HEIGHT: usize = 10;

struct SLDict {
    first: NodeRef,
}

impl Node {
    fn new(key: Option<usize>, value: String, height: usize) -> Self {
        Node {
            key,
            value,
            nexts: vec![None; height],
        }
    }
}

impl SLDict {
    pub fn new() -> Self {
        SLDict {
            first: Some(Rc::new(RefCell::new(Node::new(
                None,
                "".to_owned(),
                HEIGHT,
            )))),
        }
    }

    fn random_height(&self) -> usize {
        for level in 1..HEIGHT {
            if rand::random::<f64>() > 0.5 {
                return level;
            }
        }

        HEIGHT - 1
    }

    // Encapsulate all the borrowing in these utility methods.

    fn next_node(node: &Shared<Node>, level: usize) -> NodeRef {
        let rc = node.clone();
        let current_node = rc.borrow();
        let Some(next) = current_node.nexts.get(level).expect("Missing level") else {
            return None;
        };

        Some(next.clone())
    }

    fn get_key(node: &Shared<Node>) -> Option<usize> {
        node.borrow().key
    }

    fn get_value(node: &Shared<Node>) -> String {
        node.borrow().value.clone()
    }

    fn set_value(node: &Shared<Node>, value: &str) {
        let mut node = node.borrow_mut();
        node.value = value.to_owned();
    }

    fn set_next(node: &Shared<Node>, level: usize, next: NodeRef) {
        let mut node = node.borrow_mut();
        node.nexts[level] = next;
    }

    fn node_height(node: &Shared<Node>) -> usize {
        node.borrow().nexts.len()
    }

    fn find_predecessor(&self, key: usize, with_journey: bool) -> (NodeRef, Option<Vec<NodeRef>>) {
        let mut maybe_journey = if with_journey {
            Some(vec![None; HEIGHT])
        } else {
            None
        };
        let mut current = self.first.clone().expect("Missing first");
        for level in (0..HEIGHT).rev() {
            loop {
                if let Some(next) = SLDict::next_node(&current, level)
                    && let Some(next_key) = SLDict::get_key(&next)
                    && next_key < key
                {
                    // Keys on this level are still less than key, continue
                    current = next.clone();
                    continue;
                }

                if let Some(journey) = maybe_journey.as_mut() {
                    // No next item, or larger key. So this is the journey point for this level.
                    journey[level] = Some(current.clone());
                }
                break;
            }
        }

        (Some(current), maybe_journey)
    }

    fn get_journey_node(journey: &[NodeRef], level: usize) -> Shared<Node> {
        journey
            .get(level)
            .expect("Missing journey level")
            .clone()
            .expect("Missing journey node")
    }

    pub fn insert(&mut self, key: usize, value: String) {
        let (current, journey) = self.find_predecessor(key, true);
        let current = current.expect("Missing predecessor");
        let journey = journey.expect("Missing journey");

        if let Some(next_rc) = SLDict::next_node(&current, 0)
            && let Some(next_key) = SLDict::get_key(&next_rc)
            && next_key == key
        {
            SLDict::set_value(&next_rc, &value);
            return;
        }

        // Construct a new node with a random height tower.
        let height = self.random_height();
        let new_node = Node::new(Some(key), value, height);

        // Patch the new node in...
        let new_rc = Rc::new(RefCell::new(new_node));
        for level in 0..height {
            let journey_node = SLDict::get_journey_node(&journey, level);

            SLDict::set_next(&new_rc, level, SLDict::next_node(&journey_node, level));
            SLDict::set_next(&journey_node, level, Some(new_rc.clone()));
        }
    }

    pub fn get(&self, key: usize) -> Option<String> {
        // Starting at the highest level, whilst next is not None and less than key, move forward.
        // Move down a layer
        // Check next node key and return if match
        // None if no match

        let (current, _) = self.find_predecessor(key, false);
        let current = current.expect("Missing predecessor");

        if let Some(next_rc) = SLDict::next_node(&current, 0)
            && let Some(next_key) = SLDict::get_key(&next_rc)
            && next_key == key
        {
            return Some(SLDict::get_value(&next_rc));
        };

        None
    }

    pub fn delete(&mut self, key: usize) -> Option<String> {
        // As with insert, build a journey to the potential deletion point.
        // If the next item is the key to delete, remove it.

        let (current, journey) = self.find_predecessor(key, true);
        let current = current.expect("Missing predecessor");
        let journey = journey.expect("Missing journey");

        let delete_node = {
            let Some(next_rc) = SLDict::next_node(&current, 0) else {
                // No next node, so nothing to delete.
                return None;
            };

            let next_key = SLDict::get_key(&next_rc).expect("Unexpected sentinal");
            if next_key != key {
                // The next item was not a match for the key, nothing to delete.
                return None;
            }

            next_rc.clone()
        };

        let height = SLDict::node_height(&delete_node);

        // Remove the node to delete by bypassing it in the journey nodes.
        for level in 0..height {
            let journey_node = SLDict::get_journey_node(&journey, level);

            SLDict::set_next(&journey_node, level, SLDict::next_node(&delete_node, level));
        }

        Some(SLDict::get_value(&delete_node))
    }

    pub fn dump(&self) {
        for level in (0..HEIGHT).rev() {
            let mut current = self.first.clone().expect("Missing first");
            loop {
                if let Some(next) = SLDict::next_node(&current, level) {
                    print!("{:?} -> ", SLDict::get_key(&next));
                    current = next.clone();
                } else {
                    println!("END");
                    break;
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn simple_get() {
        let mut sl = SLDict::new();

        sl.insert(10, "v10".to_owned());

        assert_eq!(sl.get(10), Some("v10".to_owned()));
        assert_eq!(sl.get(5), None);
    }

    #[test]
    fn add_many() {
        let mut sl = SLDict::new();

        for i in 0..10 {
            sl.insert(i, format!("v{i}"));
        }

        for i in 0..10 {
            assert_eq!(sl.get(i), Some(format!("v{i}")));
        }

        sl.dump();
    }

    #[test]
    fn delete() {
        let mut sl = SLDict::new();

        for i in 0..10 {
            sl.insert(i, format!("v{i}"));
        }

        let deleted = sl.delete(5);
        assert_eq!(deleted, Some("v5".to_owned()));

        let deleted = sl.delete(5);
        assert_eq!(deleted, None);

        let deleted = sl.delete(0);
        assert_eq!(deleted, Some("v0".to_owned()));

        let deleted = sl.delete(9);
        assert_eq!(deleted, Some("v9".to_owned()));

        sl.dump();
    }

    #[test]
    fn update() {
        let mut sl = SLDict::new();

        sl.insert(10, "old".to_owned());
        assert_eq!(sl.get(10), Some("old".to_owned()));

        sl.insert(10, "new".to_owned());
        assert_eq!(sl.get(10), Some("new".to_owned()));
    }
}
