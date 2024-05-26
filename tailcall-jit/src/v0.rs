use std::collections::HashMap;

pub struct Headers(Vec<(DynamicValue, DynamicValue)>);

pub struct Http {
    pub path: DynamicValue,
    pub query: Vec<Conditional<QueryParam>>,
    pub headers: Headers,
    pub body: DynamicValue,
}

pub enum QueryParam {
    Empty,
    Param {
        key: DynamicValue,
        value: DynamicValue,
    },
}

impl TemplateContains for QueryParam {
    fn template_contains(&self, text: &str) -> bool {
        match self {
            QueryParam::Empty => false,
            QueryParam::Param { key, value } => {
                key.template_contains(text) || value.template_contains(text)
            }
        }
    }
}

impl TemplateContains for Headers {
    fn template_contains(&self, text: &str) -> bool {
        self.0
            .iter()
            .any(|(k, v)| k.template_contains(text) || v.template_contains(text))
    }
}

impl TemplateContains for Http {
    fn template_contains(&self, text: &str) -> bool {
        self.path.template_contains(text)
            || self.query.template_contains(text)
            || self.headers.template_contains(text)
            || self.body.template_contains(text)
    }
}

pub struct Grpc;

impl TemplateContains for Grpc {
    fn template_contains(&self, _text: &str) -> bool {
        todo!()
    }
}

pub struct GraphQL;

impl TemplateContains for GraphQL {
    fn template_contains(&self, _text: &str) -> bool {
        todo!()
    }
}

pub enum FieldType {
    Named(String),
    List(Box<FieldType>),
    Optional(Box<FieldType>),
}
impl FieldType {
    pub fn is_list(&self) -> bool {
        match self {
            FieldType::List(_) => true,
            FieldType::Named(_) => false,
            FieldType::Optional(inner) => inner.is_list(),
        }
    }
}

/// Unique identifier for a field in the query.
#[derive(Eq, PartialEq, Hash)]
pub struct FieldId(u64);

pub struct Field {
    pub name: String,
    pub parent_id: Option<FieldId>,
    pub type_of: FieldType,
}

pub struct FieldMap {
    map: HashMap<FieldId, Field>,
}

impl HasField for FieldMap {
    fn has_field(&self, id: &FieldId) -> bool {
        self.contains(id)
    }
}

impl FieldMap {
    pub fn new() -> Self {
        Self { map: HashMap::new() }
    }

    pub fn find_children(&self, parent: &FieldId) -> Vec<&FieldId> {
        self.map
            .iter()
            .filter_map(|(id, field)| field.parent_id.as_ref().map(|p| p == parent).map(|_| id))
            .collect()
    }

    pub fn contains(&self, field: &FieldId) -> bool {
        self.map.contains_key(field)
    }

    pub fn parent(&self, field: &FieldId) -> Option<&Field> {
        self.map
            .get(field)
            .and_then(|field| field.parent_id.as_ref())
            .and_then(|parent| self.map.get(parent))
    }

    pub fn parent_is_list(&self, field: &FieldId) -> bool {
        let mut id = field;

        while let Some(parent) = self.parent(id) {
            if parent.type_of.is_list() {
                return true;
            } else if let Some(parent) = &parent.parent_id {
                id = parent;
            } else {
                return false;
            }
        }

        return false;
    }
}

pub trait Transformer {
    fn transform(map: &mut ExecutionMap);
}

pub struct Pipe<A, B> {
    pub first: A,
    pub second: B,
}

pub struct Empty;
impl Transformer for Empty {
    fn transform(_map: &mut ExecutionMap) {}
}

impl<A: Transformer, B: Transformer> Transformer for Pipe<A, B> {
    fn transform(map: &mut ExecutionMap) {
        A::transform(map);
        B::transform(map);
    }
}

pub struct Transform;

impl Transform {
    pub fn empty() -> Empty {
        Empty
    }
}

pub enum DynamicValue {
    Literal(String),
    Template(Vec<String>),
}

impl DynamicValue {
    pub fn is_const(&self) -> bool {
        match self {
            DynamicValue::Literal(_) => true,
            DynamicValue::Template(_) => false,
        }
    }
}

impl TemplateContains for DynamicValue {
    fn template_contains(&self, text: &str) -> bool {
        match self {
            DynamicValue::Literal(_) => false,
            DynamicValue::Template(parts) => parts.iter().any(|part| part.contains(text)),
        }
    }
}

impl<A: TemplateContains> TemplateContains for Vec<A> {
    fn template_contains(&self, text: &str) -> bool {
        self.iter().any(|param| param.template_contains(text))
    }
}

trait TemplateContains {
    fn template_contains(&self, text: &str) -> bool;
}

pub enum Cond {
    T,
    F,
    And(Box<Cond>, Box<Cond>),
    Or(Box<Cond>, Box<Cond>),
    HasField(FieldId),
}

pub trait HasField {
    fn has_field(&self, id: &FieldId) -> bool;
}

impl Cond {
    pub fn eval<A: HasField>(&self, map: &A) -> bool {
        match self {
            Cond::T => true,
            Cond::F => false,
            Cond::And(a, b) => a.eval(map) && b.eval(map),
            Cond::Or(a, b) => a.eval(map) || b.eval(map),
            Cond::HasField(field) => map.has_field(field),
        }
    }

    pub fn eval_default(&self) -> bool {
        match self {
            Cond::T => true,
            Cond::F => false,
            Cond::And(a, b) => a.eval_default() && b.eval_default(),
            Cond::Or(a, b) => a.eval_default() || b.eval_default(),
            Cond::HasField(_) => false,
        }
    }
}

pub struct Conditional<A> {
    cond: Cond,
    is_true: A,
    is_false: A,
}

impl<A> Conditional<A> {
    pub fn eval<F: HasField>(&self, map: &F) -> &A {
        match &self.cond.eval(map) {
            true => &self.is_true,
            false => &self.is_false,
        }
    }

    pub fn check<F: HasField>(&self, map: &F) -> bool {
        self.cond.eval(map)
    }

    pub fn cond(&mut self, cond: Cond) {
        self.cond = cond;
    }
}

impl<A: TemplateContains> TemplateContains for Conditional<A> {
    fn template_contains(&self, text: &str) -> bool {
        self.is_true.template_contains(text) || self.is_false.template_contains(text)
    }
}

pub enum Task {
    Http(Http),
    Grpc(Grpc),
    GraphQL(GraphQL),
}

#[derive(Eq, PartialEq, Hash)]
pub struct TaskId(u64);

pub struct Node {
    pub parent: Option<TaskId>,
    pub field: FieldId,
    pub task: Task,
    pub hint_parent_is_list: bool,
}

impl TemplateContains for Task {
    fn template_contains(&self, text: &str) -> bool {
        match self {
            Task::Http(http) => http.template_contains(text),
            Task::Grpc(grpc) => grpc.template_contains(text),
            Task::GraphQL(graphql) => graphql.template_contains(text),
        }
    }
}

pub struct ExecutionMap {
    pub fields: FieldMap,
    pub tasks: HashMap<TaskId, Node>,
}

impl<'a> ExecutionMap {
    pub fn new() -> Self {
        Self { tasks: HashMap::new(), fields: FieldMap::new() }
    }

    pub fn insert(mut self, id: TaskId, node: Node) -> Self {
        self.tasks.insert(id, node);
        self
    }

    pub fn size(&self) -> usize {
        self.tasks.len()
    }
}

impl HasField for ExecutionMap {
    fn has_field(&self, id: &FieldId) -> bool {
        self.fields.has_field(id)
    }
}

pub struct MoveToRoot;
impl Transformer for MoveToRoot {
    fn transform(map: &mut ExecutionMap) {
        for (_, node) in &mut map.tasks {
            if node.task.template_contains("value") {
                node.parent = None;
            }
        }
    }
}

pub struct Expand;
impl Transformer for Expand {
    fn transform(map: &mut ExecutionMap) {
        let selection = &map.fields;
        for (_, node) in map.tasks.iter_mut() {
            match &mut node.task {
                Task::Http(http) => {
                    for q in &mut http.query {
                        if q.check(selection) {
                            q.cond(Cond::T)
                        }
                    }
                }
                Task::Grpc(_) => (),
                Task::GraphQL(_) => (),
            };
        }
    }
}

pub struct UseBulk;
impl Transformer for UseBulk {
    fn transform(map: &mut ExecutionMap) {
        for (_, node) in map.tasks.iter_mut() {
            node.hint_parent_is_list = map.fields.parent_is_list(&node.field)
        }
    }
}
