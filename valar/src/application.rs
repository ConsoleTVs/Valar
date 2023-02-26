use std::sync::Arc;

use async_trait::async_trait;
use regex::Error as RegexError;

use crate::http::FakeRequest;
use crate::http::Method;
use crate::routing::router::Compiled;
use crate::routing::router::Routable;
use crate::routing::Router;
use crate::Error;

#[async_trait]
pub trait Application: Routable<Application = Self> + Sized + Send + Sync + 'static {
    /// En entry point for your application.
    /// This is where you should create your application
    /// and return it. This is where you should also
    /// create any state or other things that you need
    /// for your application, such as a database connection.
    async fn create() -> Result<Self, Error>;

    /// Creates a fake application.
    /// The same as the create, but called when
    /// you want to create a fake application.
    ///
    /// This is useful for testing your application
    /// without having to start a server.
    ///
    /// Use this method to create fake state or
    /// other things that you need for your application.
    /// For example, a fake database connection.
    async fn create_fake() -> Result<Self, Error> {
        Self::create().await
    }

    /// Returns a fake application.
    /// This will create a new application based
    /// on the `create_fake` method.
    ///
    /// This is useful for testing your application
    /// without having to start a server.
    ///
    /// Use this method to quickly create a fake application
    /// for testing purposes.
    async fn fake() -> Result<FakeApplication<Self>, Error> {
        let app = Self::create_fake().await?;
        let fake = FakeApplication::new(app)?;

        Ok(fake)
    }

    /// Transforms an application into a fake application.
    ///
    /// This is useful for testing your application
    /// while allowing you to manually create the
    /// application.
    async fn into_fake(self) -> Result<FakeApplication<Self>, RegexError> {
        FakeApplication::new(self)
    }

    /// Returns a fake application and calls the given
    /// callback.
    ///
    /// This allows modifying the application before
    /// creating the fake application, therefore
    /// allowing mutations to it.
    async fn fake_and<F>(callback: F) -> Result<FakeApplication<Self>, Error>
    where
        F: FnOnce(&mut Self) + Send,
    {
        let mut app = Self::create_fake().await?;
        callback(&mut app);
        let fake = FakeApplication::new(app)?;

        Ok(fake)
    }
}

/// A fake application.
/// Useful for testing your application
/// without having to start a server.
pub struct FakeApplication<App: Application + Send + Sync + 'static> {
    app: Arc<App>,
    router: Router<App, Compiled>,
}

impl<App: Application + Send + Sync + 'static> FakeApplication<App> {
    /// Creates a new fake application.
    /// This will create a new application and
    /// a new router matcher.
    ///
    /// This is useful for testing your application
    /// without having to start a server.
    pub fn new(app: App) -> Result<Self, RegexError> {
        let app = Self {
            app: Arc::new(app),
            router: App::router().compile()?,
        };

        Ok(app)
    }

    /// Returns a reference to the router matcher.
    pub fn router(&self) -> &Router<App, Compiled> {
        &self.router
    }

    /// Returns a reference to the application.
    pub fn app(&self) -> &App {
        &self.app
    }

    /// Returns an Arc reference to the application.
    /// This will clone the application current Arc<T>.
    pub fn app_arc(&self) -> Arc<App> {
        self.app.clone()
    }

    /// Returns a fake GET request builder.
    /// Use this to simulate and assert responses
    /// from your application on the given requests.
    pub fn get(&self, path: &str) -> FakeRequest<App> {
        FakeRequest::new(self).method(Method::GET).path(path)
    }

    /// Returns a fake POST request builder.
    /// Use this to simulate and assert responses
    /// from your application on the given requests.
    pub fn post(&self, path: &str) -> FakeRequest<App> {
        FakeRequest::new(self).method(Method::POST).path(path)
    }

    /// Returns a fake PATCH request builder.
    /// Use this to simulate and assert responses
    /// from your application on the given requests.
    pub fn patch(&self, path: &str) -> FakeRequest<App> {
        FakeRequest::new(self).method(Method::PATCH).path(path)
    }

    /// Returns a fake PUT request builder.
    /// Use this to simulate and assert responses
    /// from your application on the given requests.
    pub fn put(&self, path: &str) -> FakeRequest<App> {
        FakeRequest::new(self).method(Method::PUT).path(path)
    }

    /// Returns a fake DELETE request builder.
    /// Use this to simulate and assert responses
    /// from your application on the given requests.
    pub fn delete(&self, path: &str) -> FakeRequest<App> {
        FakeRequest::new(self).method(Method::DELETE).path(path)
    }
}
