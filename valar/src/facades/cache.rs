use std::sync::Arc;

use crate::services::Cacheable;

pub struct Cache(Arc<Cacheable>);

impl Cache {
    pub fn new(cache: Arc<Cacheable>) -> Self {
        Self(cache)
    }
}
