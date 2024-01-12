use anyhow::{anyhow, Error};

#[derive(Clone)]
pub struct R2Address {
  pub bucket: String,
  pub path: String,
}

impl R2Address {
  pub fn from_string(str: String) -> Result<Self, Error> {
    // Get the bucket id from absolute path (it is expected that all paths starts with bucket id)
    let mut iter = str.split("/").filter(|s| !s.is_empty());

    let bucket = iter
      .next()
      .ok_or(anyhow!("Invalid path: Empty"))
      .map(|a| a.to_string())?;

    let path = iter.collect::<Vec<&str>>().join("/");

    Ok(R2Address { bucket, path })
  }
}
