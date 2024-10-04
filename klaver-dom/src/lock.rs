#[cfg(feature = "send")]
pub type Locket<T> = std::sync::Arc<std::sync::Mutex<T>>;

#[cfg(not(feature = "send"))]
pub type Locket<T> = std::rc::Rc<std::cell::RefCell<T>>;
