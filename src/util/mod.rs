pub mod image_util;

use axum::Json;
use hyper::{
  header::{ACCESS_CONTROL_ALLOW_ORIGIN, CONTENT_TYPE},
  HeaderMap, StatusCode,
};

pub fn json_response<T>(body: T) -> (StatusCode, HeaderMap, Json<T>) {
  (StatusCode::OK, cors_json_headers(), Json(body))
}

pub fn cors_json_headers() -> HeaderMap {
  let mut header = HeaderMap::new();
  header.insert(ACCESS_CONTROL_ALLOW_ORIGIN, "*".parse().unwrap());
  header.insert(
    CONTENT_TYPE,
    "application/json; charset=utf-8".parse().unwrap(),
  );
  header
}

pub fn cors_headers() -> HeaderMap {
  let mut header = HeaderMap::new();
  header.insert(ACCESS_CONTROL_ALLOW_ORIGIN, "*".parse().unwrap());
  header
}
