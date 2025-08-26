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
  extract::Path,
  response::Response,
  Error,
};
use hyper::{
  header::{
    ACCEPT_RANGES, ACCESS_CONTROL_ALLOW_ORIGIN, CONTENT_LENGTH, CONTENT_RANGE, CONTENT_TYPE, RANGE,
  },
  HeaderMap, StatusCode,
};
use rusqlite::named_params;

// #[cfg(reallink)]
// #[link(name = "cfb_decode")]
// extern "C" {
//   // fn inv_cfb(input_buf: *const u8, output: *mut u8, w: *const u32, iv: *const u8, len: usize);
//   fn key_expansion(key: *const u8, w: *mut u32);
//   // fn snapshot_video(file_url: *const c_char, snap_time: u64) -> SnapshotSt;
// }

#[cfg(reallink)]
#[link(name = "cfb_decode")]
extern "C" {
  // fn cfb_v2(w: *const u32, iv: *const u8, input_buf: *const u8, output: *mut u8, len: usize);
  fn inv_cfb_v2(w: *const u32, iv: *const u8, input_buf: *const u8, output: *mut u8, len: usize);
  #[allow(dead_code)]
  fn key_expansion(key: *const u8, w: *mut u32);
  // fn snapshot_video(file_url: *const c_char, snap_time: u64) -> SnapshotSt;
}

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

pub async fn image_stream_hander(Path((base_index, sub_dir)): Path<(u32, String)>) -> Response {
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
  let mount_config = sqlite_conn
    .query_row(sql.as_str(), named_params! {":id": base_index}, |row| {
      Ok(MountConfig {
        id: row.get_unwrap("id"),
        dir_path: row.get_unwrap(dir_path_name),
        url_prefix: row.get_unwrap("url_prefix"),
        api_version: row.get_unwrap("api_version"),
      })
    })
    .unwrap();

  let file_path = mount_config.dir_path + sub_dir_param.as_str();

  let mount_config = sqlite_conn
    .query_row(sql.as_str(), named_params! {":id": 1}, |row| {
      Ok(MountConfig {
        id: row.get_unwrap("id"),
        dir_path: row.get_unwrap(dir_path_name),
        url_prefix: row.get_unwrap("url_prefix"),
        api_version: row.get_unwrap("api_version"),
      })
    })
    .unwrap();
  let mut main_patition_path = mount_config.dir_path.clone();
  main_patition_path.push_str("/covers");
  main_patition_path.push_str(&file_path);
  
  let path = std::path::Path::new(&main_patition_path);

  let file_size = path.metadata().map_or_else(|_| 0, |m| m.len());
  let content_length = file_size;

  let extension = path.extension().unwrap().to_str().unwrap();
  let mut content_type_value = String::from("image/");
  content_type_value.push_str(extension);
  let mut header = HeaderMap::new();
  header.insert(ACCESS_CONTROL_ALLOW_ORIGIN, "*".parse().unwrap());
  header.insert(CONTENT_TYPE, content_type_value.parse().unwrap());
  header.insert(CONTENT_LENGTH, content_length.into());

  let mut response_builder = Response::builder().status(StatusCode::OK);
  let start = 0;
  let mock_stream = VideoStream::new(start, &main_patition_path);
  *response_builder.headers_mut().unwrap() = header;
  response_builder
    .body(Body::from_stream(mock_stream))
    .unwrap()
}

pub async fn video_exist(
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
    let mount_config = sqlite_conn
      .query_row(sql.as_str(), named_params! {":id": base_index}, |row| {
        Ok(MountConfig {
          id: row.get_unwrap("id"),
          dir_path: row.get_unwrap(dir_path_name),
          url_prefix: row.get_unwrap("url_prefix"),
          api_version: row.get_unwrap("api_version"),
        })
      })
      .unwrap();
    let file_path = mount_config.dir_path + sub_dir_param.as_str();
    file_path
  } else {
    let db_path_env = env::var("DEMO_VIDEO").unwrap();
    db_path_env
  };

  let path = std::path::Path::new(&file_path);

  let mut header = HeaderMap::new();
  header.insert(ACCESS_CONTROL_ALLOW_ORIGIN, "*".parse().unwrap());
  header.insert(CONTENT_LENGTH, 0.into());
  if path.exists() {
    let mut response_builder = Response::builder().status(StatusCode::OK);
    *response_builder.headers_mut().unwrap() = header;
    response_builder.body(Body::empty()).unwrap()

  } else {
    let mut response_builder = Response::builder().status(StatusCode::NOT_FOUND);
    *response_builder.headers_mut().unwrap() = header;
    response_builder.body(Body::empty()).unwrap()
  }

}


pub async fn demo_video_stream_hander(
  headers: HeaderMap,
  Path((_base_index, _sub_dir)): Path<(u32, String)>,
) -> Response {
  let file_path : String = "path_to_your_video.mp4.bin".to_string();
  let path = std::path::Path::new(&file_path);
  let file_size = path.metadata().map_or_else(|_| 0, |m| m.len());
  let range_header = headers.get(RANGE);

  println!("parse range_header: {:?}", range_header);

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

  println!("response range: {:?}, {:?}, {:?}, {:?}", start, end, content_length, part);

  let status_code = match part {
    true => StatusCode::PARTIAL_CONTENT,
    false => StatusCode::OK,
  };

  let key = "passwordpasswordpasswordpassword"; // 32 bytes key
  let iv = "2021000120210001"; // 16 bytes IV
  let mut response_builder = Response::builder().status(status_code);
  let mock_stream = CfbVideoStream::new(
    start, 
    &file_path, 
    iv.as_bytes().try_into().unwrap(), 
    key.as_bytes().try_into().unwrap());

  let mut header = HeaderMap::new();
  header.insert(ACCESS_CONTROL_ALLOW_ORIGIN, "*".parse().unwrap());
  header.insert(CONTENT_TYPE, "video/mp4".parse().unwrap());
  header.insert(CONTENT_LENGTH, content_length.into());
  header.insert(ACCEPT_RANGES, "bytes".parse().unwrap());
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

pub async fn video_stream_hander(
  headers: HeaderMap,
  Path((base_index, sub_dir)): Path<(u32, String)>,
) -> Response {
  tracing::debug!("video stream handler: base_index: {}, sub_dir: {}", base_index, sub_dir);
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
    let mount_config = sqlite_conn
      .query_row(sql.as_str(), named_params! {":id": base_index}, |row| {
        Ok(MountConfig {
          id: row.get_unwrap("id"),
          dir_path: row.get_unwrap(dir_path_name),
          url_prefix: row.get_unwrap("url_prefix"),
          api_version: row.get_unwrap("api_version"),
        })
      })
      .unwrap();
    let file_path = mount_config.dir_path + sub_dir_param.as_str();
    file_path
  } else {
    let db_path_env = env::var("DEMO_VIDEO").unwrap();
    db_path_env
  };

  let path = std::path::Path::new(&file_path);
  let file_size = path.metadata().map_or_else(|_| 0, |m| m.len());
  let range_header = headers.get(RANGE);

  tracing::debug!("parse range_header: {:?}", range_header);

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

  tracing::debug!("response range: {:?}, {:?}, {:?}, {:?}", start, end, content_length, part);

  let status_code = match part {
    true => StatusCode::PARTIAL_CONTENT,
    false => StatusCode::OK,
  };

  let mut response_builder = Response::builder().status(status_code);
  let video_stream = VideoStream::new(start, &file_path);
  // let key = "passwordpasswordpasswordpassword"; // 32 bytes key
  // let iv = "2021000120210001"; // 16 bytes IV
  // let cfb_video_stream = CfbVideoStream::new(
  //   start, &file_path, 
  //   iv.as_bytes().try_into().unwrap(), 
  //   key.as_bytes().try_into().unwrap());

  let mut header = HeaderMap::new();
  header.insert(ACCESS_CONTROL_ALLOW_ORIGIN, "*".parse().unwrap());
  header.insert(CONTENT_TYPE, "video/mp4".parse().unwrap());
  header.insert(CONTENT_LENGTH, content_length.into());
  header.insert(ACCEPT_RANGES, "bytes".parse().unwrap());
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
    .body(Body::from_stream(video_stream))
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
  fn new(start: u64, file_path: &String) -> Self {
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
    let mut buf = [0u8; 4096];
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

struct CfbVideoStream {
  file: File,
  iv: [u8; 16],
  w: [u32; 60],
  header_offset: usize,
}

impl CfbVideoStream {
  #[allow(dead_code)]
  fn new(start: u64, file_path: &String, iv:[u8; 16], pwd:[u8; 32] ) -> Self {
    // let db_path_env = env::var("DEMO_VIDEO").unwrap();
    let mut file = File::open(file_path).unwrap();

    let mut w: [u32; 60] = [0; 60];
    unsafe {
      key_expansion(pwd.as_ptr(), w.as_mut_ptr());
    }

    if start != 0 {
      let pad_start = start & 0xffff_ffff_ffff_fff0;
      let header_offset = (start - pad_start) as usize;
      if pad_start >= 16 {
        let _ = file.seek(std::io::SeekFrom::Start(pad_start - 16));
        let mut tmp_iv: [u8; 16] = [0u8; 16];
        let _ = file.read(&mut tmp_iv);
        Self { file, iv: tmp_iv, w, header_offset }
      } else {
        Self { file, iv, w, header_offset}
      }
    } else {
      Self { file, iv, w, header_offset: 0 }
    }
    // let _ = file.seek(std::io::SeekFrom::Start(start));
  }
}

impl futures_core::Stream for CfbVideoStream {
  type Item = Result<Bytes, Error>;

  fn poll_next(
    mut self: std::pin::Pin<&mut Self>,
    _: &mut std::task::Context<'_>,
  ) -> std::task::Poll<Option<Self::Item>> {
    let mut buf: [u8; 4096] = [0u8; 4096];
    let read_result = self.file.read(&mut buf);
    match read_result {
      Ok(read_len) => match read_len > 0 {

        true => {
          let mut output: [u8; 4096] = [0u8; 4096];
          unsafe {
            inv_cfb_v2(
              self.w.as_ptr(),
              self.iv.as_ptr(),
              buf.as_ptr(),
              output.as_mut_ptr(),
              read_len,
            );
          }
          self.iv = buf[read_len - 16..read_len].try_into().unwrap();

          let offset = self.header_offset;
          self.header_offset = 0;

          std::task::Poll::Ready(Some(Ok(Bytes::copy_from_slice(&output).slice(offset..read_len))))
        },
        false => std::task::Poll::Ready(None),
      },
      Err(_) => std::task::Poll::Ready(None),
    }
  }
}
