use tokio::sync::Mutex;
use tokio::sync::MutexGuard;

pub struct State<T>(Mutex<T>);

impl<T> State<T> {
    /// Creates a new state instance.
    ///
    /// # Example
    ///
    /// ```no_run
    /// use valar::state::State;
    ///
    /// let state = State::new(0);
    /// ```
    pub fn new(value: T) -> Self {
        Self(Mutex::new(value))
    }

    /// Returns a mutable reference to the underlying data.
    /// This call is asynchronous and will block the current
    /// task until it is able to acquire the lock.
    ///
    /// # Example
    ///
    /// ```no_run
    /// use tokio::runtime::Runtime;
    /// use valar::state::State;
    ///
    /// let state = State::new(0);
    ///
    /// let mut runtime = Runtime::new().unwrap();
    ///
    /// runtime.block_on(async {
    ///     let mut value = state.get().await;
    ///
    ///     assert_eq!(*value, 0);
    /// });
    /// ```
    pub async fn get(&self) -> MutexGuard<T> {
        self.0.lock().await
    }

    /// Sets the underlying data to the provided value.
    /// This call is asynchronous and will block the current
    /// task until it is able to acquire the lock.
    ///
    /// # Example
    ///
    /// ```no_run
    /// use tokio::runtime::Runtime;
    /// use valar::state::State;
    ///
    /// let state = State::new(0);
    ///
    /// let mut runtime = Runtime::new().unwrap();
    ///
    /// runtime.block_on(async {
    ///     state.set(1).await;
    ///
    ///     let value = state.clone().await;
    ///
    ///     assert_eq!(value, 1);
    /// });
    /// ```
    pub async fn set(&self, value: T) {
        let mut current = self.0.lock().await;

        *current = value;
    }

    /// Maps the current value to a new value.
    /// This call is asynchronous and will block the current
    /// task until it is able to acquire the lock.
    ///
    /// This will not give you back a mutex guard therefore
    /// it will not have to keep the lock open after the
    /// callback has been executed. This is useful if
    /// you want to do something with the value but
    /// don't want to keep the lock open nor manually
    /// unlock it / drop it.
    ///
    /// # Example
    ///
    /// ```no_run
    /// use tokio::runtime::Runtime;
    /// use valar::state::State;
    ///
    /// let state = State::new(0);
    ///
    /// let mut runtime = Runtime::new().unwrap();
    ///
    /// runtime.block_on(async {
    ///     state.map(|value| value + 1).await;
    ///
    ///     let value = state.clone().await;
    ///
    ///     assert_eq!(value, 1);
    /// });
    /// ```
    pub async fn map<F>(&self, callback: F) -> &Self
    where
        F: FnOnce(&T) -> T,
    {
        let mut current = self.0.lock().await;

        *current = callback(&current);

        self
    }
}

impl<T: Clone> State<T> {
    /// Returns a clone of the underlying data.
    /// This call is asynchronous and will block the current
    /// task until it is able to acquire the lock.
    ///
    /// # Example
    ///
    /// ```no_run
    /// use valar::state::State;
    /// use tokio::runtime::Runtime;
    ///
    /// let state = State::new(0);
    ///
    /// let mut runtime = Runtime::new().unwrap();
    ///
    /// runtime.block_on(async {
    ///     let value = state.clone().await;
    ///
    ///     assert_eq!(value, 0);
    /// });
    pub async fn clone(&self) -> T {
        self.0.lock().await.clone()
    }

    /// Maps the current value to a new value.
    /// This call is asynchronous and will block the current
    /// task until it is able to acquire the lock.
    ///
    /// This is the same as the `map` method except that it
    /// will return a clone of the new value.
    ///
    /// # Example
    ///
    /// ```no_run
    /// use tokio::runtime::Runtime;
    /// use valar::state::State;
    ///
    /// let state = State::new(0);
    ///
    /// let mut runtime = Runtime::new().unwrap();
    ///
    /// runtime.block_on(async {
    ///     let value = state.map_clone(|value| value + 1).await;
    ///
    ///     assert_eq!(value, 1);
    /// });
    /// ```
    pub async fn map_clone<F>(&self, callback: F) -> T
    where
        F: FnOnce(&T) -> T,
        T: Clone,
    {
        let mut current = self.0.lock().await;

        *current = callback(&current);

        current.clone()
    }
}
