//! Interface for stream.
//!
//! This module contains basic code to investigate and validate stream response based axum 
use std::{fs::File, io::Read};

use axum::{
  body::{Body, Bytes},
  response::Response,
  Error,
};
use hyper::{HeaderMap, StatusCode};

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

pub async fn video_stream_hander() -> Response {
  let mut response_builder = Response::builder().status(StatusCode::OK);
  let mock_stream = VideoStream::new();

  let mut header = HeaderMap::new();
  header.insert("Access-Control-Allow-Origin", "*".parse().unwrap());
  header.insert("content-type", "video/mp4".parse().unwrap());
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
  fn new() -> Self {
    let file = File::open("video-path.mp4").unwrap();
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