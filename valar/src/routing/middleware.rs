use std::future::Future;
use std::pin::Pin;
use std::slice::Iter;
use std::sync::Arc;

use async_trait::async_trait;

use crate::http::Handler as HttpHandler;
use crate::http::Request;
use crate::http::Result as HttpResult;
use crate::Application;

pub type Handler = Arc<
    dyn Fn(Request) -> Pin<Box<dyn Future<Output = HttpResult> + Send + 'static>>
        + Send
        + Sync
        + 'static,
>;

#[async_trait]
pub trait Middleware {
    async fn handle(&self, next: Handler, request: Request) -> HttpResult;
}

type SharableMiddleware = Arc<dyn Middleware + Send + Sync + 'static>;

pub struct Middlewares(Vec<SharableMiddleware>);

impl Default for Middlewares {
    fn default() -> Self {
        Self::new()
    }
}

impl Clone for Middlewares {
    fn clone(&self) -> Self {
        Self(self.0.clone())
    }
}

impl IntoIterator for Middlewares {
    type IntoIter = std::vec::IntoIter<Self::Item>;
    type Item = SharableMiddleware;

    fn into_iter(self) -> Self::IntoIter {
        self.0.into_iter()
    }
}

impl<'a> IntoIterator for &'a Middlewares {
    type IntoIter = Iter<'a, SharableMiddleware>;
    type Item = &'a SharableMiddleware;

    fn into_iter(self) -> Self::IntoIter {
        self.0.iter()
    }
}

impl Extend<SharableMiddleware> for Middlewares {
    fn extend<I>(&mut self, iter: I)
    where
        I: IntoIterator<Item = SharableMiddleware>,
    {
        self.0.extend(iter)
    }
}

impl FromIterator<SharableMiddleware> for Middlewares {
    fn from_iter<I>(iter: I) -> Self
    where
        I: IntoIterator<Item = SharableMiddleware>,
    {
        Self(iter.into_iter().collect())
    }
}

impl<'a> FromIterator<&'a Self> for Middlewares {
    fn from_iter<I>(middlewares: I) -> Self
    where
        I: IntoIterator<Item = &'a Self>,
    {
        let middlewares: Vec<_> = middlewares
            .into_iter()
            .flatten()
            .map(|middleware| middleware.clone())
            .collect();

        Self(middlewares)
    }
}

impl Middlewares {
    pub fn new() -> Self {
        Self(Vec::new())
    }

    pub fn push(&mut self, middleware: SharableMiddleware) {
        self.0.push(middleware);
    }

    pub fn wrap<App: Application + Send + Sync + 'static>(
        self,
        handler: HttpHandler<App>,
    ) -> HttpHandler<App> {
        let iterator = self.0.into_iter();
        Arc::new(move |app, request| {
            let handler = handler.clone();
            let handler: Handler = Arc::new(move |request| handler(app.clone(), request));

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
