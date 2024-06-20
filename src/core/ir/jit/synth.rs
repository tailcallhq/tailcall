use serde_json_borrow::{ObjectAsVec, Value};

use crate::core::ir::jit::model::{Children, Field};
use crate::core::ir::jit::store::{Data, Store};

pub struct Synth {
    operations: Vec<Field<Children>>,
    store: Store,
}

impl Synth {
    pub fn new(
        operations: Vec<Field<Children>>,
        store: Store,
    ) -> Self {
        Self {
            operations,
            store,
        }
    }
    pub fn synthesize(&self) -> Value {
        let mut vals = ObjectAsVec::default();

        for child in self.operations.iter() {
            let val = self.iter(child, None, None);
            if let Some(data) = vals.get_mut("data") {
                match data {
                    Value::Object(obj) => {
                        obj.insert(child.name.as_str(), val.clone());
                    }
                    _ => {
                        todo!()
                    }
                }
            }else {
                vals.insert("data", val);
            }
        }

        Value::Object(vals)
    }

    /// checks if type_of is an array and value is an array
    fn is_array(type_of: &crate::core::blueprint::Type, value: &Value) -> bool {
        type_of.is_list() == value.is_array()
    }

    #[inline]
    fn iter<'a>(
        &'a self,
        node: &'a Field<Children>,
        parent: Option<&'a Value>,
        index: Option<usize>,
    ) -> Value {
        match parent {
            Some(parent) => {
                if !Self::is_array(&node.type_of, parent) {
                    return Value::Null;
                }
                self.iter_inner(node, Some(parent))
            }
            None => {
                // we perform this check to avoid unnecessary hashing
                if node.ir.is_some() {
                    match self.store.get(&node.id) {
                        Some(val) => {
                            match val {
                                // if index is given, then the data should be a list
                                // if index is not given, then the data should be a value
                                // must return Null in all other cases.
                                Data::Value(val) => {
                                    if index.is_some() {
                                        return Value::Null;
                                    }
                                    self.iter_inner(node, Some(val))
                                }
                                Data::List(list) => {
                                    if let Some(i) = index {
                                        match list.get(i) {
                                            Some(val) => self.iter_inner(node, Some(val)),
                                            None => Value::Null
                                        }
                                    } else {
                                        Value::Null
                                    }
                                }
                            }
                        }
                        None => {
                            // IR exists, so there must be a value.
                            // if there is no value then we must return Null
                            Value::Null
                        }
                    }
                }else {
                    // either of parent value or IR must exist
                    // if none exist, then we must return Null
                    Value::Null
                }
            }
        }
        /*        match self.store.get(&node.id) {
                    Some(data) => {
                        // if index is given, then the data should be a list
                        // if index is not given, then the data should be a value
                        // must return Null in all other cases.
                        match data {
                            Data::Value(val) => {
                                if index.is_some() {
                                    return Value::Null;
                                }
                                todo!()
                            }
                            Data::List(list) => {
                                if index.is_none() {
                                    return Value::Null;
                                }
                                todo!()
                            }
                        }
                    }
                    None => {
                        if let Some(val) = parent {
                            if !Self::is_array(&node.type_of, val) {
                                return Value::Null;
                            }
                        }

                        self.iter_inner(node, parent)
                    }
                }*/
    }
    #[inline]
    fn iter_inner<'a>(
        &'a self,
        node: &'a Field<Children>,
        parent: Option<&'a Value>,
    ) -> Value {
        match parent {
            Some(Value::Object(obj)) => {
                let mut ans = ObjectAsVec::default();
                let children = node.children();

                let val = obj.get(node.name.as_str());

                if children.is_empty() {
                    // if it's a leaf node, then push the value
                    if let Some(val) = val {
                        ans.insert(node.name.as_str(), val.to_owned());
                    }else {
                        todo!("idk")
                    }
                }else {
                    for child in children {
                        let val = self.iter(child, obj.get(child.name.as_str()), None);
                        ans.insert(child.name.as_str(), val);
                    }
                }
                /*for (key, val) in obj.iter() {
                    ans.insert(
                        key, self.iter(node, Some(val), None),
                    );
                }*/
                Value::Object(ans)
            }
            Some(Value::Array(arr)) => {
                let mut ans = vec![];
                for (i, val) in arr.iter().enumerate() {
                    ans.push(
                        self.iter(node, Some(val), Some(i))
                    )
                }
                let mut object = ObjectAsVec::default();
                object.insert(node.name.as_str(), Value::Array(ans));
                Value::Object(object)
            }
            Some(val) => val.clone(), // cloning here would be cheaper than cloning whole value
            None => Value::Null,
        }
    }
}

#[cfg(test)]
mod tests {
    use serde_json_borrow::Value;
    use crate::core::blueprint::Blueprint;
    use crate::core::config::{Config, ConfigModule};
    use crate::core::ir::jit::builder::Builder;
    use crate::core::ir::jit::model::FieldId;
    use crate::core::ir::jit::store::{Data, Store};
    use crate::core::ir::jit::synth::Synth;
    use crate::core::valid::Validator;

    const POSTS: &str = r#"
        [
                {
                    "id": 1,
                    "userId": 1,
                    "title": "Some Title"
                },
                {
                    "id": 2,
                    "userId": 1,
                    "title": "Not Some Title"
                }
            ]
    "#;

    const USER1: &str = r#"
        {
            "user": {
                "id": 1,
                "name": "foo"
            }
        }
    "#;

    const USER2: &str = r#"
        {
            "user": {
                "id": 2,
                "name": "bar"
            }
        }
    "#;

    enum TestData {
        Posts,
        Users,
        User1,
        User2,
    }

    impl TestData {
        fn into_value(self) -> Data<'static> {
            match self {
                Self::Posts => Data::Value(serde_json::from_str(POSTS).unwrap()),
                Self::User1 => Data::Value(serde_json::from_str(USER1).unwrap()),
                Self::User2 => Data::Value(serde_json::from_str(USER2).unwrap()),
                TestData::Users => {
                    Data::List(vec![
                        serde_json::from_str(USER1).unwrap(),
                        serde_json::from_str(USER2).unwrap(),
                    ])
                }
            }
        }
    }

    const CONFIG: &str = include_str!("./fixtures/jsonplaceholder-mutation.graphql");

    fn foo(query: &str, store: Vec<(FieldId, Data<'static>)>) -> String {
        let doc = async_graphql::parser::parse_query(query).unwrap();
        let config = Config::from_sdl(CONFIG).to_result().unwrap();
        let config = ConfigModule::from(config);

        let builder = Builder::new(Blueprint::try_from(&config).unwrap(), doc);
        let plan = builder.build().unwrap();

        let store = store.into_iter().fold(Store::new(), |mut store, (id, data)| {
            store.insert(id, data);
            store
        });

        let synth = Synth::new(plan.into_children(), store);
        let val = synth.synthesize();

        serde_json::to_string_pretty(&val).unwrap()
    }

    #[test]
    fn test_single() {
        let store = vec![
            (FieldId::new(0), TestData::Posts.into_value()),
            (FieldId::new(1), TestData::Users.into_value()),
        ];
        /*let mut store = Store::new();
        store.insert(FieldId::new(0), Data::Value(TestData::Posts.into_value()));
        store.insert(FieldId::new(1), Data::List(
            vec![
                TestData::User1.into_value(),
                TestData::User2.into_value(),
            ]
        ));*/

        let val = foo(
            r#"
            query {
                posts { user { id } }
            }
        "#,
            store,
        );
        println!("{}", val);
    }
}