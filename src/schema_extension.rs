use std::sync::Arc;

use async_graphql::extensions::{Extension, ExtensionFactory};

#[derive(Clone)]
pub struct SchemaExtension {
    extension_factory: Arc<dyn ExtensionFactory>,
}

impl SchemaExtension {
    pub fn new(extension_factory: impl ExtensionFactory) -> Self {
        Self { extension_factory: Arc::new(extension_factory) }
    }
}

impl ExtensionFactory for SchemaExtension {
    fn create(&self) -> Arc<dyn Extension> {
        self.extension_factory.create()
    }
}
