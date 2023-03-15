use std::sync::Arc;

use async_trait::async_trait;

#[async_trait]
pub trait Application {}

/// A fake application.
/// Useful for testing your application
/// without having to start a server.
pub struct Fake<App: Application + Send + Sync + 'static>(Arc<App>);

impl<App: Application + Send + Sync + 'static> Fake<App> {
    /// Creates a new fake application. This will
    /// create a new application and a new router
    /// matcher.
    ///
    /// This is useful for testing your application
    /// without having to start a server.
    pub fn new(app: App) -> Self {
        Self(Arc::new(app))
    }
}

impl<App: Application + Send + Sync + 'static> Clone for Fake<App> {
    fn clone(&self) -> Self {
        Self(self.0.clone())
    }
}
