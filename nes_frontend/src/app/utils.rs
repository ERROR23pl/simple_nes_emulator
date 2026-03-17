// std imports
use std::collections::VecDeque;

// crates.io imports
use serde;

#[derive(serde::Deserialize, serde::Serialize)]
pub struct FixedSizeQueue<T, const L: usize>(pub VecDeque<T>);

impl<T, const L: usize> Default for FixedSizeQueue<T, L> {
    fn default() -> Self {
        FixedSizeQueue(VecDeque::new())
    }
}

impl<T, const L: usize> FixedSizeQueue<T, L> {
    pub fn push(&mut self, element: T) -> Option<T> {
        self.0.push_front(element);

        if self.0.len() > L {
            self.0.pop_back()
        } else {
            None
        }
    }
}

impl<T: PartialEq, const L: usize> FixedSizeQueue<T, L> {
    pub fn push_without_duplicates(&mut self, element: T) -> Option<T> {
        self.0.retain(|r| *r != element);
        self.push(element)
    }
}