use std::slice::Iter;
use std::sync::Arc;
use std::vec::IntoIter;

use async_trait::async_trait;

use crate::http::Request;
use crate::http::Response;
use crate::Application;

#[async_trait]
pub trait Middleware<App: Application + Send + Sync + 'static> {
    async fn before(&self, app: Arc<App>, request: &mut Request) -> Option<Response>;
    async fn after(&self, app: Arc<App>, response: &mut Response);
}

pub struct Middlewares<App: Application>(Vec<Arc<dyn Middleware<App> + Send + Sync>>);

pub struct AfterMiddlewares<App: Application>(Vec<Arc<dyn Middleware<App> + Send + Sync>>);

impl<App: Application> Extend<Arc<dyn Middleware<App> + Send + Sync>> for Middlewares<App> {
    fn extend<T: IntoIterator<Item = Arc<dyn Middleware<App> + Send + Sync>>>(&mut self, iter: T) {
        self.0.extend(iter);
    }
}

impl<App: Application> Clone for Middlewares<App> {
    fn clone(&self) -> Self {
        Self(self.0.clone())
    }
}

impl<App: Application> Default for Middlewares<App> {
    fn default() -> Self {
        Self(Vec::new())
    }
}

impl<App: Application> AfterMiddlewares<App> {
    pub fn new(middlewares: Vec<Arc<dyn Middleware<App> + Send + Sync>>) -> Self {
        Self(middlewares)
    }

    pub async fn after(&self, app: Arc<App>, response: &mut Response) {
        for middleware in self.0.iter().rev() {
            middleware.after(app.clone(), response).await;
        }
    }
}

impl<App: Application> Middlewares<App> {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn push(&mut self, middleware: Arc<dyn Middleware<App> + Send + Sync>) {
        self.0.push(middleware);
    }

    pub fn pop(&mut self) -> Option<Arc<dyn Middleware<App> + Send + Sync>> {
        self.0.pop()
    }

    pub fn len(&self) -> usize {
        self.0.len()
    }

    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }

    pub fn iter(&self) -> std::slice::Iter<'_, Arc<dyn Middleware<App> + Send + Sync>> {
        self.0.iter()
    }

    pub async fn before(
        &self,
        app: Arc<App>,
        request: &mut Request,
    ) -> (Option<Response>, AfterMiddlewares<App>) {
        let mut after: Vec<Arc<dyn Middleware<App> + Send + Sync>> = Vec::new();

        for middleware in &self.0 {
            let response = middleware.before(app.clone(), request).await;
            after.push(middleware.clone());

            if let Some(response) = response {
                return (Some(response), AfterMiddlewares::new(after));
            }
        }

        (None, AfterMiddlewares::new(after))
    }
}

impl<'a, App: Application> FromIterator<&'a Self> for Middlewares<App> {
    fn from_iter<T: IntoIterator<Item = &'a Self>>(middlewares: T) -> Self {
        let middlewares: Vec<_> = middlewares
            .into_iter()
            .flatten()
            .map(|middleware| middleware.clone())
            .collect();

        Self(middlewares)
    }
}

impl<App: Application> IntoIterator for Middlewares<App> {
    type IntoIter = IntoIter<Self::Item>;
    type Item = Arc<dyn Middleware<App> + Send + Sync>;

    fn into_iter(self) -> Self::IntoIter {
        self.0.into_iter()
    }
}

impl<'a, App: Application> IntoIterator for &'a Middlewares<App> {
    type IntoIter = Iter<'a, Arc<dyn Middleware<App> + Send + Sync>>;
    type Item = &'a Arc<dyn Middleware<App> + Send + Sync>;

    fn into_iter(self) -> Self::IntoIter {
        self.0.iter()
    }
}
