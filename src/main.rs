use axum::{body::Bytes, extract::Path, routing::{get, post}, Json, Router};
use business_handles::{add_tag, mount_config_handler, mp4_dir_handler, mp4_dir_handler1, query_tags, video_info_handler, video_rate};
use handles::{
  all_duplicate_cover, all_duplicate_video, designation_search, generate_video_snapshot, init_video_handler, parse_designation_all_handler, parse_designation_handler, parse_meta_info_all_handler, sync_mysql2sqlite_mount_config, sync_mysql2sqlite_video_info, video_detail, video_meta_info_handler, IS_LINUX, SQLITE_CONN
};
use hyper::StatusCode;
use rusqlite::Connection;
use serde_derive::{Deserialize, Serialize};
use stream_handlers::{file_stream_hander, mock_stream_hander};
use tower_http::trace::TraceLayer;
use tracing::Span;
use std::{env, ffi::{c_char, c_void, CStr, CString}, future::Future, time::Duration,};

use sysinfo::System;

mod test_main;
mod test_aes;
mod handles;
mod entity;
mod business_handles;
mod test_designation;
mod designation;
mod stream_handlers;
mod video_name_util;

#[repr(C)]
struct RustObject {
  a: i32,
  b: i32,
}

#[repr(C)]
struct CharArrObject {
  a: *const c_char,
  b: i32,
}

#[cfg(reallink)]
#[link(name = "simpledll")]
extern {
    fn simple_dll_function() -> i32;
    fn simple_dll_function_with_param(param: &RustObject) -> i32;
    fn simple_dll_function_return_struct() -> *mut RustObject;
    fn simple_dll_function_return_char_arr() -> *mut CharArrObject;
}


#[cfg(mocklink)]
fn simple_dll_function() -> i32 {
  return 0;
}

#[cfg(mocklink)]
fn simple_dll_function_with_param(_: &RustObject) -> i32 {
  return 0;
}

#[cfg(mocklink)]
fn simple_dll_function_return_struct() -> *mut RustObject {
  // return null_mut();
  let mut b_ro = Box::new(RustObject{a:1, b:2});
  let p_ro = b_ro.as_mut();
  p_ro
}

#[cfg(mocklink)]
fn simple_dll_function_return_char_arr() -> *mut CharArrObject {
  let video_name = CString::new("0").unwrap();
  let mut b_ro = Box::new(CharArrObject{a: video_name.as_ptr(), b:2});
  let p_ro = b_ro.as_mut();
  p_ro
}

#[tokio::main]
async fn main() {
  unsafe {
    let simple = simple_dll_function();
    println!("simple:{}", simple);
    let rust_obj = RustObject{a: 1, b: 1024};

    let simple = simple_dll_function_with_param(&rust_obj);
    println!("rust_obj.b:{}", rust_obj.b);
    println!("simple:{}", simple);

    if cfg!(reallink) {
      let rust_object_point = simple_dll_function_return_struct();
      println!("rust_object:{}, {}", (*rust_object_point).a, (*rust_object_point).b);
      libc::free(rust_object_point as *mut c_void);

      let char_arr_object_point = simple_dll_function_return_char_arr();
      let a_string = CString::from(CStr::from_ptr((*char_arr_object_point).a));
      
      println!("rust_object:{:?}", a_string);
      libc::free(char_arr_object_point as *mut c_void);
    }
  }
  println!("{:?}", System::name());
  let db_path_env = env::var("DB_PATH").unwrap_or_else(|_|String::from("/home/knightingal/source/keys/mp41000.db"));
  let use_mysql = env::var("USE_MYSQL").map_or_else(|_|false, |v|v == "true");
  println!("use_mysql:{}", use_mysql);
  println!("db_path:{}", db_path_env);

  let lite_conn = Box::new(Connection::open(db_path_env).unwrap());
  let is_linux = Box::new(System::name().unwrap().contains("Linux") || System::name().unwrap() == "Deepin") ;
  unsafe {
    SQLITE_CONN = Some(Box::leak(lite_conn));
    IS_LINUX = Some(Box::leak(is_linux));
  }

  let app = Router::new()
    // mantain
    .route("/", get(root))
    .route("/init-video/:base_index/*sub_dir", get(init_video_handler))
    .route("/sync-mysql2sqlite-video-info", get(sync_mysql2sqlite_video_info))
    .route("/sync-mysql2sqlite-mount-config", get(sync_mysql2sqlite_mount_config))
    .route("/users/name/:name/age/:age", post(create_user))
    .route("/parse-designation/:base_index/*sub_dir", get(parse_designation_handler))
    .route("/parse-designation-all", get(parse_designation_all_handler))
    .route("/designation-search/:designation_ori", get(designation_search))
    .route("/all-duplicate-video", get(all_duplicate_video))
    .route("/all-duplicate-cover", get(all_duplicate_cover))
    .route("/video-detail/:id", get(video_detail))
    .route("/generate-video-snapshot/*sub_dir", get(generate_video_snapshot))
    .route("/video-meta-info/*sub_dir", get(video_meta_info_handler))
    .route("/parse-meta-info-all-handler", get(parse_meta_info_all_handler))

    // bussiness
    .route("/mount-config", get(mount_config_handler))
    .route("/mp4-dir/:base_index/", get(mp4_dir_handler1))
    .route("/mp4-dir/:base_index", get(mp4_dir_handler1))
    .route("/mp4-dir/:base_index/*sub_dir", get(mp4_dir_handler))
    .route("/video-info/:base_index/*sub_dir", get(video_info_handler))
    .route("/video-rate/:id/:rate", post(video_rate))
    .route("/add-tag/:tag", post(add_tag))
    .route("/query-tags", get(query_tags))

    // demo
    .route("/mock-steam", get(mock_stream_hander))
    .route("/file-steam", get(file_stream_hander))
    .layer(TraceLayer::new_for_http().on_body_chunk(
      |chunk: &Bytes, _latency: Duration, _span: &Span| {
          tracing::debug!("streaming {} bytes", chunk.len());
      },
  ));
    // .with_state(pool)
    // ;
  let listener = tokio::net::TcpListener::bind("0.0.0.0:8082").await.unwrap();
  axum::serve(listener, app).await.unwrap();
}


fn root() -> impl Future<Output = &'static str> {
  async {
    r###################"
      .route("/init-video/:base_index/*sub_dir", get(init_video_handler))
      .route("/sync-mysql2sqlite-video-info", get(sync_mysql2sqlite_video_info))
      .route("/sync-mysql2sqlite-mount-config", get(sync_mysql2sqlite_mount_config))
      .route("/users/name/:name/age/:age", post(create_user))
      .route("/parse-designation/:base_index/*sub_dir", get(parse_designation_handler))
      .route("/parse-designation-all", get(parse_designation_all_handler))
      .route("/designation-search/:designation_ori", get(designation_search))
      .route("/all-duplicate-video", get(all_duplicate_video))
      .route("/all-duplicate-cover", get(all_duplicate_cover))
      .route("/video-detail/:id", get(video_detail))
      .route("/generate-video-snapshot/*sub_dir", get(generate_video_snapshot))
      .route("/video-meta-info/*sub_dir", get(video_meta_info_handler))
      .route("/parse-meta-info-all-handler", get(parse_meta_info_all_handler))

      .route("/mount-config", get(mount_config_handler))
      .route("/mp4-dir/:base_index/", get(mp4_dir_handler1))
      .route("/mp4-dir/:base_index", get(mp4_dir_handler1))
      .route("/mp4-dir/:base_index/*sub_dir", get(mp4_dir_handler))
      .route("/video-info/:base_index/*sub_dir", get(video_info_handler))
      .route("/video-rate/:id/:rate", post(video_rate))
      .route("/add-tag/:tag", post(add_tag))
      .route("/query-tags", get(query_tags));

      .route("/mock-steam", get(mock_stream_hander))
      
      github.com 140.82.116.4
      github.githubassets.com 185.199.110.154,185.199.111.154,185.199.109.154,185.199.108.154
      avatars.githubusercontent.com 185.199.111.133,185.199.109.133,185.199.108.133,185.199.110.133
      collector.github.com 140.82.112.22
      api.github.com 140.82.113.6
    "###################
  }
}


fn get_sqlite_connection() -> &'static Connection {
  let conn: &Connection = unsafe {
    SQLITE_CONN.unwrap()
  };
  return conn;
}


async fn create_user(Path((name,age)): Path<(String, u32)>, Json(payload): Json<CreateUser>) -> (StatusCode, Json<User>) {
  let name:String = name;
  let age: u32 = age;

  let user = User {
    id: 1337,
    age, name,
    username: payload.username
  };

  (StatusCode::CREATED, Json(user))
}



#[derive(Deserialize)]
struct CreateUser {
  username: String,
}

#[derive(Serialize)]
struct User {
  id: u64,
  age: u32,
  name: String,
  username: String,
}
