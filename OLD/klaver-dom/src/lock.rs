#[cfg(feature = "parallel")]
pub type Locket<T> = std::sync::Arc<std::sync::Mutex<T>>;

#[cfg(not(feature = "parallel"))]
pub type Locket<T> = std::rc::Rc<std::cell::RefCell<T>>;
