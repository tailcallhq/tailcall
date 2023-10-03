use hyper::header::{HeaderName, HeaderValue};
use hyper::HeaderMap;

use crate::valid::ValidationError;

pub fn validate_headers(headers: Option<Vec<(String, String)>>) -> Result<(), ValidationError<String>> {
    let mut header_map = HeaderMap::new();
    let mut errors = ValidationError::empty();
    if let Some(headers) = &headers {
        for (k, v) in headers {
            // Append header name and value to error message in case of error
            HeaderName::from_bytes(k.as_bytes()).map_err(|e| {
                errors.append(e.to_string());
            })?;

            HeaderValue::from_str(v.as_str()).
                map_err(|e| {
                    errors.append(e.to_string());
                })?;

        }
    }

    // Return validationerror if any errors are present
    if errors.is_empty() {
        Ok(())
    } else {
        Err(errors)
    }
}