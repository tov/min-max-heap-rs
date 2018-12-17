#[derive(Debug, PartialEq, Eq)]
pub struct FakeHeap<T>(Vec<T>);

impl<T: Ord> FakeHeap<T> {
    pub fn new() -> Self {
        FakeHeap(Vec::new())
    }

    fn find_min(&self) -> Option<usize> {
        self.0.iter().enumerate().min_by_key(|p| p.1).map(|p| p.0)
    }

    fn find_max(&self) -> Option<usize> {
        self.0.iter().enumerate().max_by_key(|p| p.1).map(|p| p.0)
    }

    pub fn push(&mut self, element: T) {
        self.0.push(element);
    }

    pub fn peek_min(&self) -> Option<&T> {
        self.find_min().map(|i| &self.0[i])
    }

    pub fn peek_max(&self) -> Option<&T> {
        self.find_max().map(|i| &self.0[i])
    }

    fn pop_index(&mut self, i: usize) -> T {
        let last = self.0.len() - 1;
        self.0.swap(i, last);
        self.0.pop().unwrap()
    }

    pub fn pop_min(&mut self) -> Option<T> {
        self.find_min().map(|i| self.pop_index(i))
    }

    pub fn pop_max(&mut self) -> Option<T> {
        self.find_max().map(|i| self.pop_index(i))
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

