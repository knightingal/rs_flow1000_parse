use axum::{
  Json, Router, body::Bytes, extract::Path, routing::{delete, get, post}
};
use business_handles::{
  add_tag, bind_tag, mount_config_handler, mp4_dir_handler, mp4_dir_handler1, query_tags,
  query_tags_by_video, query_videos_by_tag, statistic_handle, unbind_tag, video_info_handler,
  video_rate,
};
use handles::{
  all_duplicate_cover, all_duplicate_video, designation_search, generate_video_snapshot,
  init_video_handler, move_cover, parse_designation_all_handler, parse_designation_handler,
  parse_meta_info_all_handler, snapshot_handler, sync_mysql2sqlite_mount_config,
  sync_mysql2sqlite_video_info, video_detail, video_meta_info_handler
};
use hyper::StatusCode;
use serde_derive::{Deserialize, Serialize};
use std::{
  env,
  ffi::{c_char, c_void, CStr, CString},
  future::Future,
  time::Duration,
};
use stream_handlers::{
  file_stream_hander, image_stream_by_path_hander, mock_stream_hander, video_exist, video_stream_hander,
};
use tower_http::trace::TraceLayer;
use tracing::Span;

use sysinfo::System;

use crate::{base_lib::{init_key, linux_init}, business_handles::delete_video, handles::{cfb_video_by_id, cfb_video_by_path, parse_meta_info_by_id}, stream_handlers::{demo_video_stream_hander, image_stream_by_id_handler, video_stream_by_id_handler}};

mod business_handles;
mod designation;
mod entity;
mod handles;
mod stream_handlers;
mod test_aes;
mod test_designation;
mod test_main;
mod test_video_name_util;
mod video_name_util;
mod base_lib;
mod util;

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
extern "C" {
  fn simple_dll_function() -> i32;
  fn simple_dll_function_with_param(param: &RustObject) -> i32;
  fn simple_dll_function_return_struct() -> *mut RustObject;
  fn simple_dll_function_return_char_arr() -> *mut CharArrObject;
  fn simple_dll_function_return_heap_point() -> *const c_char;
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
  let mut b_ro = Box::new(RustObject { a: 1, b: 2 });
  let p_ro = b_ro.as_mut();
  p_ro
}

#[cfg(mocklink)]
fn simple_dll_function_return_char_arr() -> *mut CharArrObject {
  let video_name = CString::new("0").unwrap();
  let mut b_ro = Box::new(CharArrObject {
    a: video_name.as_ptr(),
    b: 2,
  });
  let p_ro = b_ro.as_mut();
  p_ro
}

#[cfg(mocklink)]
fn simple_dll_function_return_heap_point() -> *const c_char {
  let video_name = CString::new("0").unwrap();
  let heap_point = video_name.into_raw();
  heap_point
}

#[cfg(reallink)]
#[link(name = "cfb_decode")]
extern "C" {
  #[allow(dead_code)]
  fn key_expansion(key: *const u8, w: *mut u32);
}


#[tokio::main]
async fn main() {

  init_key();

  unsafe {
    let simple = simple_dll_function();
    println!("simple:{}", simple);
    let rust_obj = RustObject { a: 1, b: 1024 };

    let simple = simple_dll_function_with_param(&rust_obj);
    println!("rust_obj.b:{}", rust_obj.b);
    println!("simple:{}", simple);

    let hp = simple_dll_function_return_heap_point();
    println!("hp:{}", *hp);
    libc::free(hp as *mut c_void);

    if cfg!(reallink) {
      let rust_object_point = simple_dll_function_return_struct();
      println!(
        "rust_object:{}, {}",
        (*rust_object_point).a,
        (*rust_object_point).b
      );
      libc::free(rust_object_point as *mut c_void);

      let char_arr_object_point = simple_dll_function_return_char_arr();
      let a_string = CString::from(CStr::from_ptr((*char_arr_object_point).a));

      println!("rust_object:{:?}", a_string);
      libc::free(char_arr_object_point as *mut c_void);
    }
  }
  println!("{:?}", System::name());
  let db_path_env = env::var("DB_PATH")
    .unwrap_or_else(|_| String::from("/home/knightingal/source/keys/mp41000.db"));
  let use_mysql = env::var("USE_MYSQL").map_or_else(|_| false, |v| v == "true");
  println!("use_mysql:{}", use_mysql);
  println!("db_path:{}", db_path_env);

  linux_init();


  let app = Router::new()
    // mantain
    .route("/", get(root))
    .route("/init-video/:base_index/*sub_dir", get(init_video_handler))
    .route(
      "/sync-mysql2sqlite-video-info",
      get(sync_mysql2sqlite_video_info),
    )
    .route(
      "/sync-mysql2sqlite-mount-config",
      get(sync_mysql2sqlite_mount_config),
    )
    .route("/users/name/:name/age/:age", post(create_user))
    .route(
      "/parse-designation/:base_index/*sub_dir",
      get(parse_designation_handler),
    )
    .route("/parse-designation-all", get(parse_designation_all_handler))
    .route(
      "/designation-search/:designation_ori",
      get(designation_search),
    )
    .route("/all-duplicate-video", get(all_duplicate_video))
    .route("/all-duplicate-cover", get(all_duplicate_cover))
    .route("/video-detail/:id", get(video_detail))
    .route(
      "/generate-video-snapshot/*sub_dir",
      get(generate_video_snapshot),
    )
    .route("/snapshot/*sub_dir", get(snapshot_handler))
    .route("/video-meta-info/*sub_dir", get(video_meta_info_handler))
    .route(
      "/parse-meta-info-all-handler",
      get(parse_meta_info_all_handler),
    )
    .route(
      "/parse-meta-info-by-id/:id",
      get(parse_meta_info_by_id),
    )
    .route("/move-cover", get(move_cover))
    .route("/cfb-video-by-path/:base_index/*sub_dir", get(cfb_video_by_path))
    .route("/cfb-video-by-id/:id", get(cfb_video_by_id))
    // bussiness
    .route("/mount-config", get(mount_config_handler))
    .route("/mp4-dir/:base_index/", get(mp4_dir_handler1))
    .route("/mp4-dir/:base_index", get(mp4_dir_handler1))
    .route("/mp4-dir/:base_index/*sub_dir", get(mp4_dir_handler))
    .route("/video-info/:base_index/*sub_dir", get(video_info_handler))
    .route("/video-rate/:id/:rate", post(video_rate))
    .route("/video/:id", delete(delete_video))
    .route("/add-tag/:tag", post(add_tag))
    .route("/query-tags", get(query_tags))
    .route("/bind-tag/:tag_id/:video_id", post(bind_tag))
    .route("/unbind-tag/:tag_id/:video_id", post(unbind_tag))
    .route("/query-tags-by-video/:video_id", get(query_tags_by_video))
    .route("/query-videos-by-tag/:tag_id", get(query_videos_by_tag))
    .route("/statistic/patition/:id", get(statistic_handle))
    // demo
    .route("/mock-steam", get(mock_stream_hander))
    .route("/file-steam", get(file_stream_hander))
    .route(
      "/demo-video/:base_index/*sub_dir",
      get(demo_video_stream_hander),
    )
    .route(
      "/video-stream/:base_index/*sub_dir",
      get(video_stream_hander),
    )
    .route("/video-exist/:base_index/*sub_dir", get(video_exist))
    .route(
      "/image-stream-by-path/:base_index/*sub_dir",
      get(image_stream_by_path_hander),
    )
    .route(
      "/image-stream-by-id/:id",
      get(image_stream_by_id_handler),
    )
    .route(
      "/image-size-by-id/:id",
      get(image_size_by_id_handler),
    )
    .route(
      "/video-stream-by-id/:id/stream.mp4",
      get(video_stream_by_id_handler),
    )
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
    .route("/snapshot/*sub_dir", get(snapshot_handler))
    .route("/video-meta-info/*sub_dir", get(video_meta_info_handler))
    .route("/parse-meta-info-all-handler", get(parse_meta_info_all_handler))
    .route("/move-cover", get(move_cover))
    .route("/cfb-video-by-path/:base_index/*sub_dir", get(cfb_video_by_path))
    .route("/cfb-video-by-id/:id", get(cfb_video_by_id))
    // bussiness
    .route("/mount-config", get(mount_config_handler))
    .route("/mp4-dir/:base_index/", get(mp4_dir_handler1))
    .route("/mp4-dir/:base_index", get(mp4_dir_handler1))
    .route("/mp4-dir/:base_index/*sub_dir", get(mp4_dir_handler))
    .route("/video-info/:base_index/*sub_dir", get(video_info_handler))
    .route("/video-rate/:id/:rate", post(video_rate))
    .route("/add-tag/:tag", post(add_tag))
    .route("/query-tags", get(query_tags))
    .route("/bind-tag/:tag_id/:video_id", post(bind_tag))
    .route("/unbind-tag/:tag_id/:video_id", post(unbind_tag))
    .route("/query-tags-by-video/:video_id", get(query_tags_by_video))
    .route("/query-videos-by-tag/:tag_id", get(query_videos_by_tag))
    .route("/statistic/patition/:id", get(statistic_handle))
    // demo
    .route("/mock-steam", get(mock_stream_hander))
    .route("/file-steam", get(file_stream_hander))
    .route("/demo-video/:base_index/*sub_dir", get(demo_video_stream_hander))
    .route("/video-stream/:base_index/*sub_dir", get(video_stream_hander))
    .route("/video-exist/:base_index/*sub_dir", get(video_exist))
    .route("/image-stream/:base_index/*sub_dir", get(image_stream_hander))
      
      github.com 140.82.116.4
      github.githubassets.com 185.199.110.154,185.199.111.154,185.199.109.154,185.199.108.154
      avatars.githubusercontent.com 185.199.111.133,185.199.109.133,185.199.108.133,185.199.110.133
      collector.github.com 140.82.112.22
      api.github.com 140.82.113.6
    "###################
  }
}



async fn create_user(
  Path((name, age)): Path<(String, u32)>,
  Json(payload): Json<CreateUser>,
) -> (StatusCode, Json<User>) {
  let name: String = name;
  let age: u32 = age;

  let user = User {
    id: 1337,
    age,
    name,
    username: payload.username,
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
