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

        HEIGHT
    }

    pub fn insert(&mut self, key: usize, value: String) {
        let mut journey = vec![None; HEIGHT];
        let mut current = self.first.clone().expect("Missing first");
        for level in (0..HEIGHT).rev() {
            loop {
                let rc = current.clone();
                let current_node = rc.borrow();

                if let Some(next) = current_node.nexts.get(level).expect("Missing level")
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
            let rc = current.clone();
            let current_node = rc.borrow();
            let simple_update = if let Some(next_rc) =
                current_node.nexts.first().expect("Missing level 0")
                && let next = next_rc.borrow()
                && let Some(next_key) = next.key
                && next_key == key
            {
                true
            } else {
                false
            };

            if simple_update {
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

    pub fn get(&self, key: usize) -> Option<String> {
        // Starting at the highest, whilst next is not None and less than key, move forward.
        // Move down a layer
        // Check key and return if match
        // None if no match

        let mut current = self.first.clone().expect("Missing first");
        for level in (0..HEIGHT).rev() {
            loop {
                let rc = current.clone();
                let current_node = rc.borrow();

                if let Some(next) = current_node.nexts.get(level).expect("Missing level")
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

        let rc = current.clone();
        let current_node = rc.borrow();
        if let Some(next_rc) = current_node.nexts.first().expect("Missing level 0")
            && let next = next_rc.borrow()
            && let Some(next_key) = next.key
            && next_key == key
        {
            return Some(next.value.clone());
        };

        None
    }

    pub fn delete(&mut self, key: usize) -> bool {
        todo!()
    }

    pub fn dump(&self) {
        for level in (0..HEIGHT).rev() {
            let mut current = self.first.clone().expect("Missing first");
            loop {
                let current2 = current.clone();
                let current_node = current2.borrow();

                if let Some(Some(next)) = current_node.nexts.get(level) {
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

pub fn main() {
    todo!()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn simple_get() {
        let mut sl = SLDict::new();

        sl.insert(10, "v10".to_owned());
        println!("Get 10: {:?}", sl.get(10));
        dbg!(sl.get(5));
    }

    #[test]
    fn add_many() {
        let mut sl = SLDict::new();

        for i in 0..10 {
            sl.insert(i, format!("v{}", i));
        }

        sl.dump();
    }
}
