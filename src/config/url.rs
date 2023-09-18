use serde::Serializer;

pub fn serialize<S>(url: &Option<url::Url>, serializer: S) -> Result<S::Ok, S::Error>
where
  S: Serializer,
{
  match url {
    Some(url) => {
      let mut url_str = url.to_string();
      if url_str.ends_with('/') {
        url_str.pop();
      }
      serializer.serialize_str(&url_str)
    }
    None => serializer.serialize_none(),
  }
}
