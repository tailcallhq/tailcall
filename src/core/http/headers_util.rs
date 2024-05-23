use std::str::FromStr;

pub fn to_reqwest_headers(hyp: &hyper::header::HeaderMap) -> reqwest::header::HeaderMap {
    hyp.iter()
        .fold(reqwest::header::HeaderMap::new(), |mut map, (k, v)| {
            // the unwraps should be fine
            map.insert(
                reqwest::header::HeaderName::from_str(k.as_str()).unwrap(),
                reqwest::header::HeaderValue::from_str(v.to_str().unwrap()).unwrap(),
            );
            map
        })
}

pub fn to_hyper_headers(
    req: &reqwest::header::HeaderMap,
) -> hyper::header::HeaderMap<hyper::header::HeaderValue> {
    req.iter()
        .fold(hyper::header::HeaderMap::new(), |mut map, (k, v)| {
            if let (Ok(key), Ok(value)) = (
                hyper::header::HeaderName::from_str(k.as_str()),
                hyper::header::HeaderValue::from_str(v.to_str().unwrap()),
            ) {
                map.insert(key, value);
            }
            map
        })
}
