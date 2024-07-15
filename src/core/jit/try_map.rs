use crate::core::jit::model::{Arg, Field, Flat, Nested, OperationPlan};

pub trait TryMap<Input, Value, Output, Error> {
    fn try_map(self, map: &impl Fn(Input) -> Result<Value, Error>) -> Result<Output, Error>;
}

impl<Input, Value, Error> TryMap<Input, Value, Arg<Value>, Error> for Arg<Input> {
    fn try_map(self, map: &impl Fn(Input) -> Result<Value, Error>) -> Result<Arg<Value>, Error> {
        Ok(Arg {
            id: self.id,
            name: self.name,
            type_of: self.type_of,
            value: self.value.map(map).transpose()?,
            default_value: self.default_value.map(map).transpose()?,
        })
    }
}

impl<Input, Value, Err> TryMap<Input, Value, Field<Nested<Value>, Value>, Err>
    for Field<Nested<Input>, Input>
{
    fn try_map(
        self,
        map: &impl Fn(Input) -> Result<Value, Err>,
    ) -> Result<Field<Nested<Value>, Value>, Err> {
        let mut extensions = None;

        if let Some(nested) = self.extensions {
            let mut exts = vec![];
            for v in nested.into_inner() {
                exts.push(v.try_map(map)?);
            }
            extensions = Some(Nested::new(exts));
        }

        Ok(Field {
            id: self.id,
            name: self.name,
            ir: self.ir,
            type_of: self.type_of,
            extensions,
            skip: self.skip,
            include: self.include,
            args: self
                .args
                .into_iter()
                .map(|arg| arg.try_map(map))
                .collect::<Result<_, _>>()?,
        })
    }
}

impl<Input, Value, Err> TryMap<Input, Value, Field<Flat, Value>, Err> for Field<Flat, Input> {
    fn try_map(
        self,
        map: &impl Fn(Input) -> Result<Value, Err>,
    ) -> Result<Field<Flat, Value>, Err> {
        Ok(Field {
            id: self.id,
            name: self.name,
            ir: self.ir,
            type_of: self.type_of,
            extensions: self.extensions,
            skip: self.skip,
            include: self.include,
            args: self
                .args
                .into_iter()
                .map(|arg| arg.try_map(map))
                .collect::<Result<_, _>>()?,
        })
    }
}

impl<Input, Value, Err> TryMap<Input, Value, OperationPlan<Value>, Err> for OperationPlan<Input> {
    fn try_map(
        self,
        map: &impl Fn(Input) -> Result<Value, Err>,
    ) -> Result<OperationPlan<Value>, Err> {
        let mut flat = vec![];
        for v in self.flat {
            flat.push(v.try_map(map)?);
        }

        let mut nested = vec![];
        for v in self.nested {
            nested.push(v.try_map(map)?);
        }

        Ok(OperationPlan { flat, operation_type: self.operation_type, nested })
    }
}

#[cfg(test)]
mod tests {
    use serde_json_borrow::OwnedValue;

    use crate::core::blueprint::Blueprint;
    use crate::core::config::{Config, ConfigModule};
    use crate::core::jit::builder::Builder;
    use crate::core::jit::model::Variables;
    use crate::core::jit::try_map::TryMap;
    use crate::core::valid::Validator;
    const CONFIG: &str = include_str!("fixtures/jsonplaceholder-mutation.graphql");
    const QUERY: &str = r#"
        {
            posts {
                title
            }
        }
    "#;

    #[test]
    fn test_operation() {
        let doc = async_graphql::parser::parse_query(QUERY).unwrap();

        let config = Config::from_sdl(CONFIG).to_result().unwrap();
        let config = ConfigModule::from(config);
        let builder = Builder::new(&Blueprint::try_from(&config).unwrap(), doc);
        let plan = builder.build(&Variables::new()).unwrap();
        let plan_str = plan
            .clone()
            .try_map(&|v| Ok::<_, anyhow::Error>(v.to_string()))
            .unwrap();

        let plan = plan
            .try_map(&|v| Ok::<_, anyhow::Error>(OwnedValue::from_str(v.to_string().as_str())?))
            .unwrap();

        let plan = plan
            .clone()
            .try_map(&|v| Ok::<_, anyhow::Error>(v.to_string()))
            .unwrap();

        assert_eq!(plan_str.flat.len(), plan.flat.len());
    }
}
