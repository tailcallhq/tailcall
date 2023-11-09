use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, PartialEq, Eq, Clone, Debug)]
pub struct JsPlugin {
  pub src: String,
}

impl JsPlugin {
  pub fn merge_right(self, other: Self) -> Self {
    Self {
      src: if self.src.is_empty() {
        other.src
      } else {
        self.src
      }
    }
  }
}
