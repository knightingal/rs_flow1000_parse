
use axum::{extract::Path, routing::{get, post}, Json, Router};
use hyper::StatusCode;
use serde_derive::{Deserialize, Serialize};

mod test_main;
mod test_aes;

#[tokio::main]
async fn main() {
  let app = Router::new()
    .route("/", get(root))
    .route("/users/name/:name/age/:age", post(create_user))
    .route("/video-info/:base_index/*sub_dir", get(video_info_handler))
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

async fn video_info_handler(Path((base_index, sub_dir)): Path<(u32, String)>) -> (StatusCode, Json<DirInfo>) {
  println!("{}", base_index);
  println!("{}", sub_dir);
  let vi = VideoInfo {
    id: 1,
    video_file_name: "vid.mp4".to_string(),
    cover_file_name: "vid.jpg".to_string(),
  };

  let di = DirInfo {
    id: base_index,
    sub_dir,
    video_list: vec![vi]
  };
  (StatusCode::OK, Json(di))
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
  #[serde(alias = "videoFileName")]
  video_file_name: String,
  #[serde(alias = "coverFileName")]
  cover_file_name: String,
}

#[derive(Serialize)]
struct DirInfo {
  id: u32,
  #[serde(alias = "subDir")]
  sub_dir: String,
  #[serde(alias = "videoList")]
  video_list: Vec<VideoInfo>
}