use axum::{extract::Path, routing::{get, post}, Json, Router};
use business_handles::{mp4_dir_handler, mp4_dir_handler1, video_rate, mount_config_handler, video_info_handler,};
use handles::{
  all_duplicate_cover, all_duplicate_video, designation_search, generate_video_snapshot, init_video_handler, parse_designation_all_handler, parse_designation_handler, sync_mysql2sqlite_mount_config, sync_mysql2sqlite_video_info, video_detail, video_meta_info_handler, IS_LINUX, POOL, SQLITE_CONN
};
use hyper::StatusCode;
use mysql::{Pool, PooledConn};
use rusqlite::Connection;
use serde_derive::{Deserialize, Serialize};
use std::{env, ffi::{c_char, c_void, CStr, CString}, future::Future};

use sysinfo::System;

mod test_main;
mod test_aes;
mod handles;
mod business_handles;
mod test_designation;
mod designation;
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

#[link(name = "simpledll")]
extern {
    fn simple_dll_function() -> i32;
    fn simple_dll_function_with_param(param: &RustObject) -> i32;
    fn simple_dll_function_return_struct() -> *mut RustObject;
    fn simple_dll_function_return_char_arr() -> *mut CharArrObject;
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

    let rust_object_point = simple_dll_function_return_struct();
    println!("rust_object:{}, {}", (*rust_object_point).a, (*rust_object_point).b);
    libc::free(rust_object_point as *mut c_void);

    let char_arr_object_point = simple_dll_function_return_char_arr();
    let a_string = CString::from(CStr::from_ptr((*char_arr_object_point).a));
    
    println!("rust_object:{:?}", a_string);
    libc::free(char_arr_object_point as *mut c_void);
  }
  println!("{:?}", System::name());
  let db_path_env = env::var("DB_PATH").unwrap_or_else(|_|String::from("/home/knightingal/mp41000.db"));
  let use_mysql = env::var("USE_MYSQL").map_or_else(|_|false, |v|v == "true");
  println!("use_mysql:{}", use_mysql);
  println!("db_path:{}", db_path_env);

  if use_mysql {
    let url = "mysql://root:000000@localhost:3306/mp4viewer";
    // let pool = Pool::new(url).unwrap();
    let box_pool = Box::new(Pool::new(url).unwrap());
    unsafe {
      POOL = Some(Box::leak(box_pool));
    }
  }
  let lite_conn = Box::new(Connection::open(db_path_env).unwrap());
  let is_linux = Box::new(System::name().unwrap().contains("Linux"));
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

    // bussiness
    .route("/mount-config", get(mount_config_handler))
    .route("/mp4-dir/:base_index/", get(mp4_dir_handler1))
    .route("/mp4-dir/:base_index", get(mp4_dir_handler1))
    .route("/mp4-dir/:base_index/*sub_dir", get(mp4_dir_handler))
    .route("/video-info/:base_index/*sub_dir", get(video_info_handler))
    .route("/video-rate/:id/:rate", post(video_rate));
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

      .route("/mount-config", get(mount_config_handler))
      .route("/mp4-dir/:base_index/", get(mp4_dir_handler1))
      .route("/mp4-dir/:base_index", get(mp4_dir_handler1))
      .route("/mp4-dir/:base_index/*sub_dir", get(mp4_dir_handler))
      .route("/video-info/:base_index/*sub_dir", get(video_info_handler))
      .route("/video-rate/:id/:rate", post(video_rate));
    "###################
  }
}


fn get_sqlite_connection() -> &'static Connection {
  let conn: &Connection = unsafe {
    SQLITE_CONN.unwrap()
  };
  return conn;
}

fn get_mysql_connection() -> PooledConn {
  let conn = unsafe {
      POOL.unwrap().get_conn().unwrap()
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


/* 
fn sample_code_list() -> Vec<&'static str> {
  return vec![
    "MIDD",
    "MDED",
    "MIRD",
    "MIGD",
    "MIID",
    "MIAD",
    "MIBD",
    "MIMK",
    "ASS",
    "ES",
    "NEW",
    "REPLAY",
    "LEGEND",
    "MINT",
    "ONED",
    "SOE",
    "SPS",
    "ONSD",
    "KIRD",
    "BLK",
    "KISD",
    "GG",
    "JJ",
    "KK",
    "SCOP",
    "TBL",
    "MZQ",
    "YSN",
    "DXMN",
    "LABS",
    "AM",
    "BF",
    "SUPD",
    "NSS",
    "ATOM",
    "BDD",
    "ARSO",
    "FAA",
    "SW",
    "NGD",
    "TBL",
    "HBAD",
    "TMDI",
    "DCS",
    "CWM",
    "OKAD",
    "MVBD",
    "MVSD",
    "SUNS",
    "UMD",
    "MOMJ",
    "TARD",
    "HUNT",
    "DVDES",
    "ROY",
    "SASS",
    "OLS",
    "ATT",
    "INF",
    "DCM",
    "MN",
    "AGEMIX",
    "BDSR",
    "WDI",
    "WSS",
    "NATR",
    "MAST",
    "ONCE",
    "WOBB",
    "ODFR",
    "ODFW",
    "APAD",
    "APAR",
    "SERO",
    "DXN",
    "HUNT",
    "GAR",
    "SVDVD",
    "RCT",
    "NGKS",
    "RD",
    "KUF",
    "IPTD",
    "IPZIPZ",
    "IDBD",
    "SUPD",
    "IPSD",
    "SVND",
    "HBAD",
    "MV",
    "VSPDS",
    "VSPDR",
    "FSET",
    "DANDY",
    "LADY",
    "SVDVD",
    "NMD",
    "UFD",
    "CXD",
    "BBI",
    "BEB",
    "NST",
    "BUR",
    "FTA",
    "NEO",
    "CRPD",
    "JUKD",
    "JUC",
    "ATID",
    "RBD",
    "JBD",
    "SHKD",
    "SSPD",
    "MDYD",
    "PGD",
    "PJD",
    "WANZ",
    "KAWD",
    "KAPD",
    "MXGS",
    "MX3DS",
    "MXSPS",
    "DDT",
    "STAR",
    "SACE",
    "SDDM",
    "SDDE",
    "SDMT",
    "OVDES",
    "NHDTA",
    "IESP",
    "IDOL",
    "IENE",
    "OPEN",
    "FSDSS",
  ];
}
*/

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
