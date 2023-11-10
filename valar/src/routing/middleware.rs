use std::future::Future;
use std::pin::Pin;
use std::slice::Iter;
use std::sync::Arc;

use async_trait::async_trait;

use crate::http::Handler as HttpHandler;
use crate::http::Request;
use crate::http::Result as HttpResult;

pub type Handler<App> = Arc<
    dyn Fn(Request<App>) -> Pin<Box<dyn Future<Output = HttpResult> + Send + 'static>>
        + Send
        + Sync
        + 'static,
>;

#[async_trait]
pub trait Middleware<App: Send + Sync + 'static> {
    async fn handle(&self, next: Handler<App>, request: Request<App>) -> HttpResult;
}

type SharableMiddleware<App> = Arc<dyn Middleware<App> + Send + Sync + 'static>;

pub struct Middlewares<App: Send + Sync + 'static>(Vec<SharableMiddleware<App>>);

impl<App: Send + Sync + 'static> Default for Middlewares<App> {
    fn default() -> Self {
        Self::new()
    }
}

impl<App: Send + Sync + 'static> Clone for Middlewares<App> {
    fn clone(&self) -> Self {
        Self(self.0.clone())
    }
}

impl<App: Send + Sync + 'static> IntoIterator for Middlewares<App> {
    type IntoIter = std::vec::IntoIter<Self::Item>;
    type Item = SharableMiddleware<App>;

    fn into_iter(self) -> Self::IntoIter {
        self.0.into_iter()
    }
}

impl<'a, App: Send + Sync + 'static> IntoIterator for &'a Middlewares<App> {
    type IntoIter = Iter<'a, SharableMiddleware<App>>;
    type Item = &'a SharableMiddleware<App>;

    fn into_iter(self) -> Self::IntoIter {
        self.0.iter()
    }
}

impl<App: Send + Sync + 'static> Extend<SharableMiddleware<App>> for Middlewares<App> {
    fn extend<I>(&mut self, iter: I)
    where
        I: IntoIterator<Item = SharableMiddleware<App>>,
    {
        self.0.extend(iter)
    }
}

impl<App: Send + Sync + 'static> FromIterator<SharableMiddleware<App>> for Middlewares<App> {
    fn from_iter<I>(iter: I) -> Self
    where
        I: IntoIterator<Item = SharableMiddleware<App>>,
    {
        Self(iter.into_iter().collect())
    }
}

impl<'a, App: Send + Sync + 'static> FromIterator<&'a Self> for Middlewares<App> {
    fn from_iter<I>(middlewares: I) -> Self
    where
        I: IntoIterator<Item = &'a Self>,
    {
        let middlewares: Vec<_> = middlewares.into_iter().flatten().cloned().collect();

        Self(middlewares)
    }
}

impl<App: Send + Sync + 'static> Middlewares<App> {
    pub fn new() -> Self {
        Self(Vec::new())
    }

    pub fn push(&mut self, middleware: SharableMiddleware<App>) {
        self.0.push(middleware);
    }

    pub fn wrap(self, handler: HttpHandler<App>) -> HttpHandler<App> {
        let iterator = self.0.into_iter();
        Arc::new(move |request| {
            let handler = handler.clone();
            // let handler: Handler<App> = Arc::new(move |request|
            // handler(request));

            let handler = iterator
                .clone()
                .rev()
                .fold(handler, move |next, middleware| {
                    Arc::new(move |request| {
                        let next = next.clone();
                        let middleware = middleware.clone();
                        Box::pin(async move { middleware.handle(next, request).await })
                    })
                });

            Box::pin(async move { handler(request).await })
        })
    }
}
