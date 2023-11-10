pub trait Config<T> {
    fn config(&self) -> T;
}
