use std::sync::Arc;

pub struct ProtoGeneratorConfig {
    query: String,
    mutation: String,
    is_mutation: Arc<dyn Fn(String) -> bool>,
}

impl ProtoGeneratorConfig {
    pub fn new(query: Option<String>, mutation: Option<String>) -> Self {
        Self {
            query: query.unwrap_or("Query".to_string()),
            mutation: mutation.unwrap_or("Mutation".to_string()),
            is_mutation: Arc::new(|_| false),
        }
    }

    pub fn is_mutation(&self, name: String) -> bool {
        (self.is_mutation)(name)
    }
    pub fn query(&self) -> &str {
        self.query.as_str()
    }

    pub fn mutation(&self) -> &str {
        self.mutation.as_str()
    }
}

impl Default for ProtoGeneratorConfig {
    fn default() -> Self {
        Self {
            query: "Query".to_string(),
            mutation: "Mutation".to_string(),
            is_mutation: Arc::new(|_| false),
        }
    }
}
