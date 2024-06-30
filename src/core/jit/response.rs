pub struct Response<Value, Error> {
    pub data: Value,
    pub errors: Vec<Error>,
    pub extensions: Vec<(String, Value)>,
}

impl<Value, Error> Response<Value, Error> {
    pub fn new(_result: Result<Value, Error>) -> Self {
        todo!()
    }
}
