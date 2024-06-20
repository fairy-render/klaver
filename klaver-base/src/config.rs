use std::path::PathBuf;

pub trait Getter<T> {
    fn getter(&self) -> T;
}

impl<T, F> Getter<T> for F
where
    F: Fn() -> T,
{
    fn getter(&self) -> T {
        (self)()
    }
}

#[derive(Default)]
pub struct Config {
    cwd: Option<Box<dyn Getter<Option<PathBuf>>>>,
    env: Option<Box<dyn Getter<PathBuf>>>,
    args: Option<Box<dyn Getter<Vec<String>>>>,
}

impl Config {
    pub fn set_cwd<T>(&mut self, func: T)
    where
        T: Getter<Option<PathBuf>> + 'static,
    {
        self.cwd = Some(Box::new(func))
    }

    pub fn get_cwd(&self) -> Option<PathBuf> {
        self.cwd.as_ref().and_then(|m| m.getter())
    }

    pub fn set_args<T>(&mut self, func: T)
    where
        T: Getter<Vec<String>> + 'static,
    {
        self.args = Some(Box::new(func))
    }

    pub fn get_args(&self) -> Option<Vec<String>> {
        self.args.as_ref().map(|m| m.getter())
    }
}
