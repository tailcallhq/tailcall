use std::sync::Arc;

use async_graphql::extensions::{Extension, ExtensionFactory};

#[derive(Clone)]
pub struct SchemaExtension(Arc<dyn ExtensionFactory>);

impl SchemaExtension {
    pub fn new(extension_factory: impl ExtensionFactory) -> Self {
        Self(Arc::new(extension_factory))
    }
}

impl ExtensionFactory for SchemaExtension {
    fn create(&self) -> Arc<dyn Extension> {
        self.0.create()
    }
}
