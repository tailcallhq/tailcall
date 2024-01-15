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
  Cond(Vec<(Box<ExprBody>, Box<ExprBody>)>),
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
