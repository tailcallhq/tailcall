use tailcall::Expression;

mod tailcall {
    pub struct Blueprint {}
    pub struct Expression {}
}

mod async_graphql {
    pub struct SelectionSet {}
}

struct TypeInfo {}

struct FieldInfo {
    execute: ExecutionPlan,
    type_info: TypeInfo,
}

struct Field<A> {
    name: String,
    selection: Selection<A>,
    info: A,
}

struct Selection<A> {
    fields: Vec<Field<A>>,
}

struct Blueprint<A> {
    selection: Selection<A>,
}

impl<A> Blueprint<A> {
    fn generate(&self, blueprint: tailcall::Blueprint) -> Blueprint<FieldInfo> {
        todo!()
    }
}


struct DataLoader {}

enum ExecutionPlan {
    Par(Box<ExecutionPlan>, Box<ExecutionPlan>),
    Seq(Box<ExecutionPlan>, Box<ExecutionPlan>),
    Many(Box<ExecutionPlan>),
    Single(tailcall::Expression),
    GraphQL(Box<ExecutionPlan>, async_graphql::SelectionSet),
    Batch(Box<ExecutionPlan>, DataLoader),
}

/// Optimizes the execution plan.
trait Optimizer {
    fn optimize(&self, plan: ExecutionPlan) -> ExecutionPlan;
}

/// Finds all plans that don't depend on the result of the previous plan and moves them to the top.
struct MoveUp {}
impl Optimizer for MoveUp {
    fn optimize(&self, plan: ExecutionPlan) -> ExecutionPlan {
        todo!()
    }
}

fn main() {
    println!("Hello, world!");
}
