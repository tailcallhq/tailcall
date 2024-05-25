use std::collections::BTreeMap;

pub struct DynamicValue;
impl DynamicValue {
    pub fn is_const() -> bool {
        todo!()
    }
}

struct Headers(BTreeMap<DynamicValue, DynamicValue>);

pub enum RPC {
    Http(Http),
    Grpc(Grpc),
    GraphQL(GraphQL),
    JS(JS),
}

pub struct QueryParam {
    name: DynamicValue,
    value: DynamicValue,
}

struct Http {
    base_url: DynamicValue,
    path: DynamicValue,
    query: Vec<QueryParam>,
    body: DynamicValue,
    headers: Headers,
}

struct Grpc {
    base_url: DynamicValue,
    method: String,
    headers: Headers,
    body: DynamicValue,
}

struct GraphQL {
    base_url: DynamicValue,
    headers: Headers,
    operation: GraphQLOperation,
}

struct GraphQLOperation {
    operation: Operation,
    name: Option<String>,
    variables: BTreeMap<String, DynamicValue>,
    selection: Vec<Selection>,
}

enum Selection {
    Field { name: String },
}

enum Operation {
    Query,
    Mutation,
}

struct JS {
    name: String,
}

impl PartialEq for RPC {
    fn eq(&self, other: &Self) -> bool {
        todo!()
    }
}

impl RPC {
    pub fn depends_on(&self, other: &Self) -> bool {
        todo!()
    }
}
