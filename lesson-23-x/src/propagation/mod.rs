use std::rc::Rc;
use std::cell::RefCell;

mod shared_propagation;

use self::shared_propagation::SharedPropagation;

pub struct Propagation {
    shared: Rc<RefCell<SharedPropagation>>,
}