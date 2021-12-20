use std::collections::btree_map::BTreeMap;

#[derive(Debug, Eq, PartialEq)]
pub struct FakeHeap<T> {
    tree: BTreeMap<T, usize>,
    len: usize,
}

impl<T: Clone + Ord> FakeHeap<T> {
    pub fn new() -> Self {
        Self {
            tree: BTreeMap::new(),
            len: 0,
        }
    }

    pub fn len(&self) -> usize {
        self.len
    }

    #[allow(dead_code)]
    pub fn is_empty(&self) -> bool {
        self.len == 0
    }

    pub fn push(&mut self, element: T) {
        if let Some(found) = self.tree.get_mut(&element) {
            *found += 1;
        } else {
            self.tree.insert(element, 0);
        }

        self.len += 1;
    }

    pub fn peek_min(&self) -> Option<&T> {
        self.tree.range(..).next().map(|p| p.0)
    }

    pub fn peek_max(&self) -> Option<&T> {
        self.tree.range(..).next_back().map(|p| p.0)
    }

    pub fn pop_min(&mut self) -> Option<T> {
        if let Some((elem, count)) = self.tree.range_mut(..).next() {
            let elem = elem.clone();
            if let Some(pred) = count.checked_sub(1) {
                *count = pred;
            } else {
                self.tree.remove(&elem); 
            }

            self.len -= 1;
            Some(elem)
        } else {
            None
        }
    }

    pub fn pop_max(&mut self) -> Option<T> {
        if let Some((elem, count)) = self.tree.range_mut(..).next_back() {
            let elem = elem.clone();
            if let Some(pred) = count.checked_sub(1) {
                *count = pred;
            } else {
                self.tree.remove(&elem); 
            }

            self.len -= 1;
            Some(elem)
        } else {
            None
        }
    }

    pub fn push_pop_min(&mut self, element: T) -> T {
        self.push(element);
        self.pop_min().unwrap()
    }

    pub fn push_pop_max(&mut self, element: T) -> T {
        self.push(element);
        self.pop_max().unwrap()
    }

    pub fn replace_min(&mut self, element: T) -> Option<T> {
        let result = self.pop_min();
        self.push(element);
        result
    }

    pub fn replace_max(&mut self, element: T) -> Option<T> {
        let result = self.pop_max();
        self.push(element);
        result
    }
}
