use serde_json::Value;

struct Context<'a> {
    value: &'a Value,
}

impl<'a> Context<'a> {
    pub fn new(value: &'a Value) -> Self {
        Self { value }
    }

    pub fn len(&self) -> usize {
        match self.value {
            Value::Array(list) => list.len(),
            _ => 0,
        }
    }
}

struct Expand<'a> {
    context: &'a Context<'a>,
}

impl<'a> Expand<'a> {
    pub fn new(context: &'a Context) -> Self {
        Self { context }
    }

    // whenever we find the list, we expand the list to match the context length.
    pub fn expand(&self, value: &mut Value) {
        match value {
            Value::Object(map) => {
                map.values_mut().for_each(|v| self.expand(v));
            }
            Value::Array(list) => {
                let length = self.context.len();
                let mut final_ans = Vec::with_capacity(length);
                for _ in 0..length {
                    final_ans.extend(list.clone());
                }
                *list = final_ans
            }
            _ => {} // do nothing in other variants.
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn main() {
        // Option 1:
        let values = Value::Array(vec![
            "1".into(),
            "2".into(),
            "3".into(),
            "4".into(),
            "5".into(),
        ]);

        // Test Option 1
        let mut input1 = json!({
            "a": { "b": { "c": { "d": ["{{.value.userId}}"] } } }
        });
        let ctx = Context::new(&values);
        Expand::new(&ctx).expand(&mut input1);
        println!("[Finder]: expanded: {:#?}", input1);
        println!("[Finder]: context: {:#?}", values);

        // Option 2:
        let values = Value::Array(vec![
            json!({
                "id": 1,
                "name": "John Doe",
                "email": "john@doe.com"
            }),
            json!({
                "id": 2,
                "name": "Jane Doe",
                "email": "jane@doe.com"
            }),
        ]);
        let mut input1 = json!([{ "userId": "{{.value.id}}", "title": "{{.value.name}}","content": "Hello World" }]);
        let ctx = Context::new(&values);
        Expand::new(&ctx).expand(&mut input1);
        println!("[Finder]: expanded: {:#?}", input1);
        println!("[Finder]: context: {:#?}", values);

        // Option 3:
        let mut input1 = json!([{ "metadata": "xyz", "items": "{{.value.userId}}" }]);
        let ctx = Context::new(&values);
        Expand::new(&ctx).expand(&mut input1);
        println!("[Finder]: expanded: {:#?}", input1);
        println!("[Finder]: context: {:#?}", values);

        // Option 4:
        let mut input1 =
            json!({ "metadata": "xyz", "items": [{"key": "id", "value": "{{.value.userId}}" }]} );
        let ctx = Context::new(&values);
        Expand::new(&ctx).expand(&mut input1);
        println!("[Finder]: expanded: {:#?}", input1);
        println!("[Finder]: context: {:#?}", values);
    }
}
