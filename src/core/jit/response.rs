use derive_setters::Setters;

#[derive(Setters)]
pub struct Response<Value, Error> {
    pub data: Option<Value>,
    pub errors: Vec<Error>,
    pub extensions: Vec<(String, Value)>,
}

impl<Value, Error> Response<Value, Error> {
    pub fn new(result: Result<Value, Error>) -> Self {
        match result {
            Ok(value) => Response {
                data: Some(value),
                errors: Vec::new(),
                extensions: Vec::new(),
            },
            Err(errors) => Response { data: None, errors: vec![errors], extensions: Vec::new() },
        }
    }
}
