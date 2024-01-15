use serde::{Deserialize, Serialize};
use serde_json::Value;

use super::{GraphQL, Grpc, Http};

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
pub struct Expr {
  pub body: ExprBody,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
pub enum ExprBody {
  #[serde(rename = "http")]
  Http(Http),
  #[serde(rename = "grpc")]
  Grpc(Grpc),
  #[serde(rename = "graphQL")]
  GraphQL(GraphQL),
  #[serde(rename = "const")]
  Const(Value),
  #[serde(rename = "if")]
  If {
    cond: Box<ExprBody>,
    #[serde(rename = "then")]
    on_true: Box<ExprBody>,
    #[serde(rename = "else")]
    on_false: Box<ExprBody>,
  },
  #[serde(rename = "concat")]
  Concat(Vec<ExprBody>),
  #[serde(rename = "intersection")]
  Intersection(Vec<ExprBody>),
}
