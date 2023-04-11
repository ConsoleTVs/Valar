pub mod cache;

pub trait Facade<T> {
    fn facade() -> T;
}
