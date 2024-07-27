use axum::{extract::Path, routing::{get, post}, Json, Router};
use handles::{
  all_duplicate_video, designation_search, init_video_handler, mount_config_handler, mp4_dir_handler, mp4_dir_handler1, parse_designation_all_handler, parse_designation_handler, sync_mysql2sqlite_mount_config, sync_mysql2sqlite_video_info, video_detail, video_info_handler, video_rate, POOL, SQLITE_CONN
};
use hyper::StatusCode;
use mysql::{Pool, PooledConn};
use rusqlite::Connection;
use serde_derive::{Deserialize, Serialize};
use std::env;

mod test_main;
mod test_aes;
mod handles;
mod test_designation;
mod designation;
mod video_name_util;


#[tokio::main]
async fn main() {
  let home_param = env::var("HOME").unwrap();
  let use_mysql_env = env::var("USE_MYSQL");
  let db_path_env = env::var("DB_PATH").unwrap();
  let mut use_mysql: bool = false;
  match use_mysql_env {
    Ok(val) => {
      println!("use_mysql:{}", val);
      use_mysql = val == "true";
    },
    Err(e) => println!("not found:{e}"),
  }
  println!("use_mysql:{}", use_mysql);
  println!("home:{}", home_param);
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
  unsafe {
    SQLITE_CONN = Some(Box::leak(lite_conn));
  }

  let app = Router::new()
    .route("/", get(root))
    .route("/sync-mysql2sqlite-video-info", get(sync_mysql2sqlite_video_info))
    .route("/sync-mysql2sqlite-mount-config", get(sync_mysql2sqlite_mount_config))
    .route("/users/name/:name/age/:age", post(create_user))
    .route("/video-info/:base_index/*sub_dir", get(video_info_handler))
    .route("/parse-designation/:base_index/*sub_dir", get(parse_designation_handler))
    .route("/parse-designation-all", get(parse_designation_all_handler))
    .route("/mount-config", get(mount_config_handler))

    .route("/mp4-dir/:base_index/", get(mp4_dir_handler1))
    .route("/mp4-dir/:base_index", get(mp4_dir_handler1))
    .route("/mp4-dir/:base_index/*sub_dir", get(mp4_dir_handler))
    .route("/init-video/:base_index/*sub_dir", get(init_video_handler))

    .route("/designation-search/:designation_ori", get(designation_search))
    .route("/all-duplicate-video", get(all_duplicate_video))
    .route("/video-detail/:id", get(video_detail))
    .route("/video-rate/:id/:rate", post(video_rate));
    // .with_state(pool)
    // ;
  let listener = tokio::net::TcpListener::bind("0.0.0.0:8082").await.unwrap();
  axum::serve(listener, app).await.unwrap();
}


async fn root() -> &'static str {
  let conn: &Connection = get_sqlite_connection();
  conn.execute("create table test_table  (id integer primary key)", ()).unwrap();

  "Hello World!"
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
