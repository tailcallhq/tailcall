use derive_setters::Setters;

#[derive(Setters)]
pub struct ProtoGeneratorConfig {
    query: String,
    mutation: String,
    is_mutation_fxn: Box<dyn Fn(&str) -> bool>,
}

impl ProtoGeneratorConfig {
    pub fn new(
        query: Option<String>,
        mutation: Option<String>,
        is_mutation_fxn: Box<dyn Fn(&str) -> bool>,
    ) -> Self {
        Self {
            query: query.unwrap_or("Query".to_string()),
            mutation: mutation.unwrap_or("Mutation".to_string()),
            is_mutation_fxn,
        }
    }

    pub fn is_mutation(&self, name: &str) -> bool {
        (self.is_mutation_fxn)(name)
    }
    pub fn get_query(&self) -> &str {
        self.query.as_str()
    }

    pub fn get_mutation(&self) -> &str {
        self.mutation.as_str()
    }
}

impl Default for ProtoGeneratorConfig {
    fn default() -> Self {
        Self {
            query: "Query".to_string(),
            mutation: "Mutation".to_string(),
            is_mutation_fxn: Box::new(|_| false),
        }
    }
}
