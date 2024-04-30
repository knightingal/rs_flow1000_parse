use axum::{extract::{Path, State}, routing::{get, post}, Json, Router};
use hyper::StatusCode;
use mysql::{params, prelude::Queryable, Pool};
use serde_derive::{Deserialize, Serialize};

mod test_main;
mod test_aes;

static mut POOL: Option<&Pool>= None;

#[tokio::main]
async fn main() {

  let url = "mysql://root:000000@localhost:3306/mp4viewer";
  let pool = Pool::new(url).unwrap();
  let box_pool = Box::new(Pool::new(url).unwrap());
  unsafe {
    POOL = Some(Box::leak(box_pool))
  }

  let app = Router::new()
    .route("/", get(root))
    .route("/users/name/:name/age/:age", post(create_user))
    .route("/video-info/:base_index/*sub_dir", get(video_info_handler))
    .route("/video-detail/:id", get(video_detail))
    .with_state(pool)
    ;
  let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();
  axum::serve(listener, app).await.unwrap();
}

async fn root() -> &'static str {
  "Hello World!"
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


async fn video_detail(State(pool): State<Pool>, Path(id): Path<u32>) -> (StatusCode, Json<VideoEntity>) {
  let mut conn1 = pool.get_conn().unwrap();
  let selected_video = conn1.exec_map(
    "select id, video_file_name, cover_file_name from video_info where id = :id ", params! {
      "id" => id,
    }, |(id, video_file_name, cover_file_name)| {VideoEntity{id, video_file_name, cover_file_name}}).unwrap();

  (StatusCode::OK, Json(selected_video.get(0).unwrap().clone()))

}

async fn video_info_handler(Path((base_index, sub_dir)): Path<(u32, String)>) 
    -> (StatusCode, Json<Vec<VideoEntity>>) {
  println!("{}", base_index);
  println!("{}", sub_dir);
  let mut sub_dir_param = String::from("/");
  sub_dir_param += &sub_dir;


  let mut conn = unsafe {
    POOL.unwrap().get_conn().unwrap()
  };

  let selected_video: Vec<VideoEntity> = conn.exec_map(
    "select id, video_file_name, cover_file_name from video_info where dir_path = :dir_path and base_index=:base_index", params! {
      "dir_path" => sub_dir_param,
      "base_index" => base_index,
    }, |(id, video_file_name, cover_file_name)| {VideoEntity{id, video_file_name, cover_file_name}}).unwrap();


  (StatusCode::OK, Json(selected_video))
}
#[derive(Serialize, Clone)]
struct VideoEntity {
  id: u32,
  #[serde(rename = "videoFileName")]
  video_file_name: String,
  #[serde(rename = "coverFileName")]
  cover_file_name: String
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

#[derive(Serialize)]
struct VideoInfo {
  id: u32,
  #[serde(rename = "videoFileName")]
  video_file_name: String,
  #[serde(rename = "coverFileName")]
  cover_file_name: String,
}

#[derive(Serialize)]
struct DirInfo {
  id: u32,
  #[serde(rename = "subDir")]
  sub_dir: String,
  #[serde(rename = "videoList")]
  video_list: Vec<VideoEntity>
}