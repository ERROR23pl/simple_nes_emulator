use std::rc::Rc;
use std::cell::RefCell;

pub type Shared<T> = Rc<RefCell<T>>;

pub trait SharedReference<T> {
    fn new(value: T) -> Self;
}


impl<T> SharedReference<T> for Shared<T> {
    fn new(value: T) -> Self {
        Rc::new(RefCell::new(value))
    }
}