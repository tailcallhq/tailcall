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
        let flat = self
            .flat
            .into_iter()
            .map(|field| field.try_map(&map))
            .collect::<Result<_, _>>()?;

        let nested = self
            .nested
            .into_iter()
            .map(|field| field.try_map(&map))
            .collect::<Result<_, _>>()?;

        Ok(OperationPlan { flat, operation_type: self.operation_type, nested })
    }
}
