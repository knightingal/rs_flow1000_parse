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
  extract::{Path, Request},
  response::Response,
  Error,
};
use hyper::{
  header::{ACCESS_CONTROL_ALLOW_ORIGIN, CONTENT_LENGTH, CONTENT_RANGE, CONTENT_TYPE, RANGE},
  HeaderMap, StatusCode,
};
use rusqlite::named_params;

use crate::{entity::MountConfig, get_sqlite_connection, handles::IS_LINUX};

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

pub async fn video_stream_hander(      headers: HeaderMap,
  Path((base_index, sub_dir)): Path<(u32, String)>,
) -> Response {

  let file_path = if sub_dir.len() > 0 {
  let mut sub_dir_param = String::from("/");
  sub_dir_param += &sub_dir;
  if sub_dir_param.ends_with("/") {
    sub_dir_param.truncate(sub_dir_param.len() - 1);
  }
  let sqlite_conn = get_sqlite_connection();
  let mut sql = String::from("select id, ");
  let dir_path_name: &str;
  unsafe {
    dir_path_name = if *IS_LINUX.unwrap() {
      "dir_path"
    } else {
      "win_dir_path"
    }
  }
  sql += dir_path_name;
  sql += " , url_prefix, api_version from mp4_base_dir where id = :id";
  let mount_config = sqlite_conn.query_row(sql.as_str(), named_params! {":id": base_index}, |row| {
      Ok(MountConfig {
        id: row.get_unwrap("id"),
        dir_path: row.get_unwrap(dir_path_name),
        url_prefix: row.get_unwrap("url_prefix"),
        api_version: row.get_unwrap("api_version"),
      })
    }).unwrap();
  let file_path = mount_config.dir_path + sub_dir_param.as_str();
  file_path
  } else {
  let db_path_env = env::var("DEMO_VIDEO").unwrap();
  db_path_env
  };

  let path = std::path::Path::new(&file_path);
  let file_size = path.metadata().map_or_else(|_| 0, |m| m.len());
  let range_header = headers.get(RANGE);

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
  let mock_stream = VideoStream::new(start, file_path);

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
  fn new(start: u64, file_path: String) -> Self {
    // let db_path_env = env::var("DEMO_VIDEO").unwrap();
    let mut file = File::open(file_path).unwrap();
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
