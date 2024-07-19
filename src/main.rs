use axum::{extract::Path, routing::{get, post}, Json, Router};
use handles::{all_duplicate_video, designation_search, mount_config_handler, mp4_dir_handler, mp4_dir_handler1, parse_designation_all_handler, parse_designation_handler, video_detail, video_info_handler, video_rate, MountConfig, VideoEntity, POOL, SQLITE_CONN};
use hyper::{HeaderMap, StatusCode};
use mysql::{prelude::Queryable, Pool};
use rusqlite::{params, Connection};
use serde_derive::{Deserialize, Serialize};

mod test_main;
mod test_aes;
mod handles;
mod test_designation;
mod designation;


#[tokio::main]
async fn main() {

  let url = "mysql://root:000000@localhost:3306/mp4viewer";
  let pool = Pool::new(url).unwrap();
  let box_pool = Box::new(Pool::new(url).unwrap());
  let lite_conn = Box::new(Connection::open("flow1000.db").unwrap());
  unsafe {
    POOL = Some(Box::leak(box_pool));
    SQLITE_CONN = Some(Box::leak(lite_conn));
  }

  let app = Router::new()
    .route("/", get(root))
    .route("/sync-mysql2sqlite", get(sync_mysql2sqlite))
    .route("/users/name/:name/age/:age", post(create_user))
    .route("/video-info/:base_index/*sub_dir", get(video_info_handler))
    .route("/parse-designation/:base_index/*sub_dir", get(parse_designation_handler))
    .route("/parse-designation-all", get(parse_designation_all_handler))
    .route("/mount-config", get(mount_config_handler))

    .route("/sync-mysql2sqlite-mount-config", get(sync_mysql2sqlite_mount_config))
    .route("/mp4-dir/:base_index/", get(mp4_dir_handler1))
    .route("/mp4-dir/:base_index", get(mp4_dir_handler1))
    .route("/mp4-dir/:base_index/*sub_dir", get(mp4_dir_handler))

    .route("/designation-search/:designation_ori", get(designation_search))
    .route("/all-duplicate-video", get(all_duplicate_video))
    .route("/video-detail/:id", get(video_detail))
    .route("/video-rate/:id/:rate", post(video_rate))
    .with_state(pool)
    ;
  let listener = tokio::net::TcpListener::bind("0.0.0.0:8082").await.unwrap();
  axum::serve(listener, app).await.unwrap();
}

async fn sync_mysql2sqlite_mount_config() -> (StatusCode, HeaderMap, Json<Vec<MountConfig>>) {
  let mut conn = unsafe {
    POOL.unwrap().get_conn().unwrap()
  };
  let sqlite_conn = unsafe {
    SQLITE_CONN.unwrap()
  };

  let mount_config: Vec<MountConfig> = conn.query_map(
    "select id, dir_path,url_prefix,api_version from mp4_base_dir ", 
    |(id, dir_path,url_prefix,api_version)| {
      return MountConfig{
        id, 
        dir_path,
        url_prefix,
        api_version
      };
    }).unwrap();


  (&mount_config).into_iter().for_each(|video_entity| {
    sqlite_conn.execute("insert into mp4_base_dir (
      id, dir_path,url_prefix,api_version
    ) values (
      ?1, ?2, ?3, ?4
    )", 
    params![video_entity.id, video_entity.dir_path, video_entity.url_prefix, video_entity.api_version]).unwrap();
  });


  let mut header = HeaderMap::new();
  header.insert("Access-Control-Allow-Origin", "*".parse().unwrap());
  header.insert("content-type", "application/json; charset=utf-8".parse().unwrap());

  (StatusCode::OK, header, Json(mount_config))
}



async fn root() -> &'static str {
  let conn: &Connection = get_sqlite_connection();
  conn.execute("create table test_table  (id integer primary key)", ()).unwrap();

  "Hello World!"
}

async fn sync_mysql2sqlite() -> (StatusCode, HeaderMap, Json<Vec<VideoEntity>>) {
  
  let mut conn = unsafe {
    POOL.unwrap().get_conn().unwrap()
  };
  let sqlite_conn = unsafe {
    SQLITE_CONN.unwrap()
  };

  let selected_video: Vec<VideoEntity> = conn.query_map(
    "select id, dir_path,base_index,rate, video_file_name, cover_file_name, designation_num,designation_char from video_info ", 
    |(id, dir_path,base_index,rate, video_file_name, cover_file_name, designation_num,designation_char)| {

      return VideoEntity{
        id, 
        video_file_name, 
        cover_file_name, 
        designation_char, 
        designation_num,
        dir_path,
        base_index,
        rate
      };
    }).unwrap();


  (&selected_video).into_iter().for_each(|video_entity| {
    sqlite_conn.execute("insert into video_info (id, dir_path,base_index,rate, video_file_name, cover_file_name, designation_num,designation_char) 
    values (?1, ?2,?3,?4,?5,?6,?7,?8)", params![video_entity.id, 
    video_entity.dir_path, video_entity.base_index, 
    video_entity.rate, video_entity.video_file_name, video_entity.cover_file_name, video_entity.designation_num, video_entity.designation_char]).unwrap();
  });


  let mut header = HeaderMap::new();
  header.insert("Access-Control-Allow-Origin", "*".parse().unwrap());
  header.insert("content-type", "application/json; charset=utf-8".parse().unwrap());

  (StatusCode::OK, header, Json(selected_video))
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
