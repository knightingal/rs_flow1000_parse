//! Interface for stream.
//!
//! This module contains basic code to investigate and validate stream response based axum
use std::{
  env,
  fs::File,
  io::{Read, Seek},
};

use axum::{
  body::{Body, Bytes},
  extract::Request,
  response::Response,
  Error,
};
use hyper::{
  header::{ACCESS_CONTROL_ALLOW_ORIGIN, CONTENT_LENGTH, CONTENT_RANGE, CONTENT_TYPE, RANGE},
  HeaderMap, StatusCode,
};

pub async fn mock_stream_hander() -> Response {
  let response_builder = Response::builder().status(StatusCode::OK);
  let mock_stream = MockStream::new();

  response_builder
    .body(Body::from_stream(mock_stream))
    .unwrap()
}

pub async fn file_stream_hander() -> Response {
  let response_builder = Response::builder().status(StatusCode::OK);
  let mock_stream = FileStream::new();

  response_builder
    .body(Body::from_stream(mock_stream))
    .unwrap()
}

pub async fn video_stream_hander(req: Request) -> Response {
  let db_path_env = env::var("DEMO_VIDEO").unwrap();
  let path = std::path::Path::new(&db_path_env);
  let file_size = path.metadata().map_or_else(|_| 0, |m| m.len());
  // let file_size = 100000u64;
  let range_header = req.headers().get(RANGE);

  let (start, end, content_length, part) = match range_header {
    Some(range_header) => {
      let (_, value) = range_header.to_str().unwrap().split_once("=").unwrap();

      let (start, end) = value.split_once("-").unwrap();
      let start: u64 = start.parse().unwrap();
      let end: u64 = end.parse().unwrap_or(file_size - 1);
      (start, end, file_size - start, true)
    }
    _ => (0, file_size - 1, file_size, false),
  };

  let status_code = match part {
    true => StatusCode::PARTIAL_CONTENT,
    false => StatusCode::OK,
  };
  let mut response_builder = Response::builder().status(status_code);
  let mock_stream = VideoStream::new(start);

  let mut header = HeaderMap::new();
  header.insert(ACCESS_CONTROL_ALLOW_ORIGIN, "*".parse().unwrap());
  header.insert(CONTENT_TYPE, "video/mp4".parse().unwrap());
  header.insert(CONTENT_LENGTH, content_length.into());
  if part {
    header.insert(
      CONTENT_RANGE,
      format!("bytes {}-{}/{}", start, end, file_size)
        .parse()
        .unwrap(),
    );
  }
  *response_builder.headers_mut().unwrap() = header;
  response_builder
    .body(Body::from_stream(mock_stream))
    .unwrap()
}

struct MockStream {
  done: bool,
}

impl MockStream {
  fn new() -> Self {
    MockStream { done: false }
  }
}

impl futures_core::Stream for MockStream {
  type Item = Result<Bytes, Error>;
  fn poll_next(
    mut self: std::pin::Pin<&mut Self>,
    _: &mut std::task::Context<'_>,
  ) -> std::task::Poll<Option<Self::Item>> {
    if self.done {
      return std::task::Poll::Ready(None);
    } else {
      self.done = true;
      let buf = Bytes::from_static(b"hello");
      return std::task::Poll::Ready(Some(Ok(buf)));
    }
  }
}

struct FileStream {
  file: File,
}

impl FileStream {
  fn new() -> Self {
    let file = File::open("/home/knightingal/source/jflow1000-server/src/main/java/org/nanking/knightingal/controller/ApkConfigController.java").unwrap();
    FileStream { file }
  }
}

impl futures_core::Stream for FileStream {
  type Item = Result<Bytes, Error>;

  fn poll_next(
    mut self: std::pin::Pin<&mut Self>,
    _: &mut std::task::Context<'_>,
  ) -> std::task::Poll<Option<Self::Item>> {
    let mut buf = [0u8; 1024];
    let read_result = self.file.read(&mut buf);
    match read_result {
      Ok(read_len) => match read_len > 0 {
        true => std::task::Poll::Ready(Some(Ok(Bytes::copy_from_slice(&buf).slice(0..read_len)))),
        false => std::task::Poll::Ready(None),
      },
      Err(_) => std::task::Poll::Ready(None),
    }
  }
}

struct VideoStream {
  file: File,
}

impl VideoStream {
  fn new(start: u64) -> Self {
    let db_path_env = env::var("DEMO_VIDEO").unwrap();
    let mut file = File::open(db_path_env).unwrap();
    let _ = file.seek(std::io::SeekFrom::Start(start));
    Self { file }
  }
}

impl futures_core::Stream for VideoStream {
  type Item = Result<Bytes, Error>;

  fn poll_next(
    mut self: std::pin::Pin<&mut Self>,
    _: &mut std::task::Context<'_>,
  ) -> std::task::Poll<Option<Self::Item>> {
    let mut buf = [0u8; 1024];
    let read_result = self.file.read(&mut buf);
    match read_result {
      Ok(read_len) => match read_len > 0 {
        true => std::task::Poll::Ready(Some(Ok(Bytes::copy_from_slice(&buf).slice(0..read_len)))),
        false => std::task::Poll::Ready(None),
      },
      Err(_) => std::task::Poll::Ready(None),
    }
  }
}
