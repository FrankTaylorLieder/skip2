#![allow(dead_code)]

// TODO: Refactor to remove duplicate code.

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

    pub fn insert(&mut self, key: usize, value: String) {
        let mut journey = vec![None; HEIGHT];
        let mut current = self.first.clone().expect("Missing first");
        for level in (0..HEIGHT).rev() {
            loop {
                if let Some(next) = SLDict::next_node(&current, level)
                    && let Some(next_key) = next.borrow().key
                    && next_key < key
                {
                    // Keys on this level are still less than key, continue
                    current = next.clone();
                    continue;
                }

                // No next item, or larger key. So this is the journey point for this level.
                journey[level] = Some(current.clone());
                break;
            }
        }

        {
            // Scope this so we drop the borrow to the current node which is also the last element
            // of the journey.
            let simple_update = if let Some(next_rc) = SLDict::next_node(&current, 0)
                && let next = next_rc.borrow()
                && let Some(next_key) = next.key
                && next_key == key
            {
                true
            } else {
                false
            };

            if simple_update {
                let rc = current.clone();
                let current_node = rc.borrow();
                let next = current_node
                    .nexts
                    .first()
                    .expect("Missing level 0")
                    .clone()
                    .expect("Missing simple update");

                next.borrow_mut().value = value;
                return;
            }
        }

        // Construct a new node with a random height tower.
        let height = self.random_height();
        let mut new_node = Node::new(Some(key), value, height);

        // Patch the new node in... note we must first update the new node's nexts before we can
        // build the NodeRef to update the journey nodes nexts. Otherwise we'll have issues with
        // ownership of the new_node and it's ref.
        for level in 0..height {
            let journey_node = journey
                .get(level)
                .expect("Missing journey level")
                .clone()
                .expect("Missing journey node");

            new_node.nexts[level] = journey_node.borrow().nexts[level].clone();
        }

        let new_ref = Rc::new(RefCell::new(new_node));
        for level in 0..height {
            let journey_node = journey
                .get(level)
                .expect("Missing journey level")
                .clone()
                .expect("Missing journey node");

            journey_node.borrow_mut().nexts[level] = Some(new_ref.clone());
        }
    }

    // Return a NodeRef of the next node at level.
    // This method needs to borrow the passed in node.
    fn next_node(node: &Shared<Node>, level: usize) -> NodeRef {
        let rc = node.clone();
        let current_node = rc.borrow();
        let Some(next) = current_node.nexts.get(level).expect("Missing level") else {
            return None;
        };

        Some(next.clone())
    }

    pub fn get(&self, key: usize) -> Option<String> {
        // Starting at the highest level, whilst next is not None and less than key, move forward.
        // Move down a layer
        // Check next node key and return if match
        // None if no match

        let mut current = self.first.clone().expect("Missing first");
        for level in (0..HEIGHT).rev() {
            loop {
                if let Some(next) = SLDict::next_node(&current, level)
                    && let Some(next_key) = next.borrow().key
                    && next_key < key
                {
                    // Keys on this level are still less than key, continue
                    current = next.clone();
                    continue;
                }

                // No next item, or larger key.
                break;
            }
        }

        if let Some(next_rc) = SLDict::next_node(&current, 0)
            && let next = next_rc.borrow()
            && let Some(next_key) = next.key
            && next_key == key
        {
            return Some(next.value.clone());
        };

        None
    }

    pub fn delete(&mut self, key: usize) -> Option<String> {
        // As with insert, build a journey to the potential deletion point.
        // If the next item is the key to delete, remove it.

        let mut journey = vec![None; HEIGHT];
        let mut current = self.first.clone().expect("Missing first");
        for level in (0..HEIGHT).rev() {
            loop {
                if let Some(next) = SLDict::next_node(&current, level)
                    && let Some(next_key) = next.borrow().key
                    && next_key < key
                {
                    // Keys on this level are still less than key, continue
                    current = next.clone();
                    continue;
                }

                // No next item, or larger key. So this is the journey point for this level.
                journey[level] = Some(current.clone());
                break;
            }
        }

        let delete_node = {
            // Scope this so we drop the borrow to the current node which is also the last element
            // of the journey.
            let Some(next_rc) = SLDict::next_node(&current, 0) else {
                // No next node, so nothing to delete.
                return None;
            };

            let next = next_rc.borrow();
            let next_key = next.key.expect("Unexpected sentinal");
            if next_key != key {
                // The next item was not a match for the key, nothing to delete.
                return None;
            }

            next_rc.clone()
        };

        let delete_node = delete_node.borrow();
        let delete_nexts = &delete_node.nexts;
        let height = delete_nexts.len();

        // Remove the node to delete by bypassing it in the journey nodes.
        (0..height).for_each(|level| {
            let journey_node = journey
                .get(level)
                .expect("Missing journey level")
                .clone()
                .expect("Missing journey node");

            journey_node.borrow_mut().nexts[level] = delete_nexts[level].clone();
        });

        Some(delete_node.value.clone())
    }

    pub fn dump(&self) {
        for level in (0..HEIGHT).rev() {
            let mut current = self.first.clone().expect("Missing first");
            loop {
                if let Some(next) = SLDict::next_node(&current, level) {
                    print!("{:?} -> ", next.borrow().key);
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
            sl.insert(i, format!("v{}", i));
        }

        for i in 0..10 {
            assert_eq!(sl.get(i), Some(format!("v{}", i)));
        }

        sl.dump();
    }

    #[test]
    fn delete() {
        let mut sl = SLDict::new();

        for i in 0..10 {
            sl.insert(i, format!("v{}", i));
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
