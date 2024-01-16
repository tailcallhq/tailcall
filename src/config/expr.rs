use serde::{Deserialize, Serialize};
use serde_json::Value;

use super::{GraphQL, Grpc, Http};

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
pub struct Expr {
  pub body: ExprBody,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
pub enum ExprBody {
  // IO
  #[serde(rename = "http")]
  Http(Http),
  #[serde(rename = "grpc")]
  Grpc(Grpc),
  #[serde(rename = "graphQL")]
  GraphQL(GraphQL),
  #[serde(rename = "const")]
  Const(Value),
  // Logic
  #[serde(rename = "if")]
  If {
    cond: Box<ExprBody>,
    #[serde(rename = "then")]
    on_true: Box<ExprBody>,
    #[serde(rename = "else")]
    on_false: Box<ExprBody>,
  },
  #[serde(rename = "allPass")]
  AllPass(Vec<ExprBody>),
  #[serde(rename = "and")]
  And(Box<ExprBody>, Box<ExprBody>),
  #[serde(rename = "anyPass")]
  AnyPass(Vec<ExprBody>),
  #[serde(rename = "cond")]
  Cond(Box<ExprBody>, Vec<(Box<ExprBody>, Box<ExprBody>)>),
  #[serde(rename = "defaultTo")]
  DefaultTo(Box<ExprBody>, Box<ExprBody>),
  #[serde(rename = "isEmpty")]
  IsEmpty(Box<ExprBody>),
  #[serde(rename = "not")]
  Not(Box<ExprBody>),
  #[serde(rename = "or")]
  Or(Box<ExprBody>, Box<ExprBody>),

  // List
  #[serde(rename = "concat")]
  Concat(Vec<ExprBody>),

  // Relation
  #[serde(rename = "intersection")]
  Intersection(Vec<ExprBody>),
  #[serde(rename = "difference")]
  Difference(Vec<ExprBody>, Vec<ExprBody>),
  #[serde(rename = "eq")]
  Equals(Box<ExprBody>, Box<ExprBody>),
  #[serde(rename = "gt")]
  Gt(Box<ExprBody>, Box<ExprBody>),
  #[serde(rename = "gte")]
  Gte(Box<ExprBody>, Box<ExprBody>),
  #[serde(rename = "lt")]
  Lt(Box<ExprBody>, Box<ExprBody>),
  #[serde(rename = "lte")]
  Lte(Box<ExprBody>, Box<ExprBody>),
  #[serde(rename = "max")]
  Max(Vec<ExprBody>),
  #[serde(rename = "min")]
  Min(Vec<ExprBody>),
  #[serde(rename = "pathEq")]
  PathEq(Box<ExprBody>, Vec<String>, Box<ExprBody>),
  #[serde(rename = "propEq")]
  PropEq(Box<ExprBody>, String, Box<ExprBody>),
  #[serde(rename = "sortPath")]
  SortPath(Vec<ExprBody>, Vec<String>),
  #[serde(rename = "symmetricDifference")]
  SymmetricDifference(Vec<ExprBody>, Vec<ExprBody>),
  #[serde(rename = "union")]
  Union(Vec<ExprBody>, Vec<ExprBody>),

  // Math
  #[serde(rename = "mod")]
  Mod(Box<ExprBody>, Box<ExprBody>),
  #[serde(rename = "add")]
  Add(Box<ExprBody>, Box<ExprBody>),
  #[serde(rename = "dec")]
  Dec(Box<ExprBody>),
  #[serde(rename = "divide")]
  Divide(Box<ExprBody>, Box<ExprBody>),
  #[serde(rename = "inc")]
  Inc(Box<ExprBody>),
  #[serde(rename = "multiply")]
  Multiply(Box<ExprBody>, Box<ExprBody>),
  #[serde(rename = "negate")]
  Negate(Box<ExprBody>),
  #[serde(rename = "product")]
  Product(Vec<ExprBody>),
  #[serde(rename = "subtract")]
  Subtract(Box<ExprBody>, Box<ExprBody>),
  #[serde(rename = "sum")]
  Sum(Vec<ExprBody>),
}

impl ExprBody {
  ///
  /// Performs a deep check on if the expression has any IO.
  ///
  pub fn has_io(&self) -> bool {
    match self {
      ExprBody::Http(_) => true,
      ExprBody::Grpc(_) => true,
      ExprBody::GraphQL(_) => true,
      ExprBody::Const(_) => false,
      ExprBody::If { cond, on_true, on_false } => cond.has_io() || on_true.has_io() || on_false.has_io(),
      ExprBody::AllPass(l) => l.iter().any(|e| e.has_io()),
      ExprBody::And(expr1, expr2) => expr1.has_io() || expr2.has_io(),
      ExprBody::AnyPass(l) => l.iter().any(|e| e.has_io()),
      ExprBody::Cond(default, branches) => {
        default.has_io() || branches.iter().any(|(cond, expr)| cond.has_io() || expr.has_io())
      }
      ExprBody::DefaultTo(expr1, expr2) => expr1.has_io() || expr2.has_io(),
      ExprBody::IsEmpty(expr) => expr.has_io(),
      ExprBody::Not(expr) => expr.has_io(),
      ExprBody::Or(expr1, expr2) => expr1.has_io() || expr2.has_io(),
      ExprBody::Concat(l) => l.iter().any(|e| e.has_io()),
      ExprBody::Intersection(l) => l.iter().any(|e| e.has_io()),
      ExprBody::Mod(expr1, expr2) => expr1.has_io() || expr2.has_io(),
      ExprBody::Add(expr1, expr2) => expr1.has_io() || expr2.has_io(),
      ExprBody::Dec(expr) => expr.has_io(),
      ExprBody::Divide(expr1, expr2) => expr1.has_io() || expr2.has_io(),
      ExprBody::Inc(expr) => expr.has_io(),
      ExprBody::Multiply(expr1, expr2) => expr1.has_io() || expr2.has_io(),
      ExprBody::Negate(expr) => expr.has_io(),
      ExprBody::Product(l) => l.iter().any(|e| e.has_io()),
      ExprBody::Subtract(expr1, expr2) => expr1.has_io() || expr2.has_io(),
      ExprBody::Sum(l) => l.iter().any(|e| e.has_io()),
      ExprBody::Difference(l1, l2) => l1.iter().any(|e| e.has_io()) || l2.iter().any(|e| e.has_io()),
      ExprBody::Equals(expr1, expr2) => expr1.has_io() || expr2.has_io(),
      ExprBody::Gt(expr1, expr2) => expr1.has_io() || expr2.has_io(),
      ExprBody::Gte(expr1, expr2) => expr1.has_io() || expr2.has_io(),
      ExprBody::Lt(expr1, expr2) => expr1.has_io() || expr2.has_io(),
      ExprBody::Lte(expr1, expr2) => expr1.has_io() || expr2.has_io(),
      ExprBody::Max(l) => l.iter().any(|e| e.has_io()),
      ExprBody::Min(l) => l.iter().any(|e| e.has_io()),
      ExprBody::PathEq(expr1, _, expr2) => expr1.has_io() || expr2.has_io(),
      ExprBody::PropEq(expr1, _, expr2) => expr1.has_io() || expr2.has_io(),
      ExprBody::SortPath(l, _) => l.iter().any(|e| e.has_io()),
      ExprBody::SymmetricDifference(l1, l2) => l1.iter().any(|e| e.has_io()) || l2.iter().any(|e| e.has_io()),
      ExprBody::Union(l1, l2) => l1.iter().any(|e| e.has_io()) || l2.iter().any(|e| e.has_io()),
    }
  }
}
