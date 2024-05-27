use std::{cmp::Ordering, fs::{self, DirEntry}};

use axum::{extract::{Path, State}, Json};
use hyper::{HeaderMap, StatusCode};
use mysql::{params, prelude::Queryable, Pool};
use serde_derive::Serialize;

pub static mut POOL: Option<&Pool>= None;
pub async fn video_detail(State(pool): State<Pool>, Path(id): Path<u32>) -> (StatusCode, Json<VideoEntity>) {
  let mut conn1 = pool.get_conn().unwrap();
  let selected_video = conn1.exec_map(
    "select id, video_file_name, cover_file_name from video_info where id = :id ", params! {
      "id" => id,
    }, |(id, video_file_name, cover_file_name)| {VideoEntity{id, video_file_name, cover_file_name}}).unwrap();

  (StatusCode::OK, Json(selected_video.get(0).unwrap().clone()))
}

pub async fn mount_config_handler()  
    -> (StatusCode, HeaderMap, Json<Vec<MountConfig>>) {
  let mut conn = unsafe {
    POOL.unwrap().get_conn().unwrap()
  };
  let mount_config_list:Vec<MountConfig> = conn.query_map(
    "select id, dir_path, url_prefix, api_version from mp4_base_dir", 
    |(id, dir_path, url_prefix, api_version)| {
      MountConfig{id, dir_path, url_prefix, api_version}
    }
  ).unwrap();

  let mut header = HeaderMap::new();
  header.insert("Access-Control-Allow-Origin", "*".parse().unwrap());
  header.insert("content-type", "application/json; charset=utf-8".parse().unwrap());

  (StatusCode::OK, header, Json(mount_config_list))
}

pub async fn mp4_dir_handler1(Path(base_index): Path<u32>) 
    -> (StatusCode, HeaderMap, Json<Vec<String>>) {
  println!("{}", base_index);

  let mut conn = unsafe {
    POOL.unwrap().get_conn().unwrap()
  };

  let dir_path: String = conn.exec_first(
    "select dir_path from mp4_base_dir where id = :id ", params! {
      "id" => base_index,
    }).unwrap().unwrap();

  let file_names = parse_dir_path(&dir_path).unwrap();

  let mut header = HeaderMap::new();
  header.insert("Access-Control-Allow-Origin", "*".parse().unwrap());
  header.insert("content-type", "application/json; charset=utf-8".parse().unwrap());

  (StatusCode::OK, header, Json(file_names))
}

pub async fn mp4_dir_handler(Path((base_index, sub_dir)): Path<(u32, String)>) 
    -> (StatusCode, HeaderMap, Json<Vec<String>>) {
  println!("{}", base_index);
  println!("{}", sub_dir);
  let mut sub_dir_param = String::from("/");
  sub_dir_param += &sub_dir;


  let mut conn = unsafe {
    POOL.unwrap().get_conn().unwrap()
  };

  let mut dir_path: String = conn.exec_first(
    "select dir_path from mp4_base_dir where id = :id ", params! {
      "id" => base_index,
    }).unwrap().unwrap();

  dir_path += "/";
  dir_path += &sub_dir;

  let file_names = parse_dir_path(&dir_path);
  let mut header = HeaderMap::new();
  header.insert("Access-Control-Allow-Origin", "*".parse().unwrap());
  header.insert("content-type", "application/json; charset=utf-8".parse().unwrap());

  (StatusCode::OK, header, Json(file_names.unwrap()))
}

fn parse_dir_path(dir_path: &String) -> Result<Vec<String>, std::io::Error> {
  let mut file_entry_list: Vec<DirEntry> = fs::read_dir(dir_path)?
    .map(|res| res.unwrap())
    .filter(|res| !res.file_name().into_string().unwrap().ends_with(".torrent")).collect();
  file_entry_list.sort_by(|a, b| comp_path(&b, &a).unwrap());

  let file_names:Vec<String> = file_entry_list.into_iter().map(|res| 
      res
        .file_name()
        .into_string()
        .unwrap()
  ).collect();

  return Result::Ok(file_names);
}

fn comp_path(a: &DirEntry, b: &DirEntry) -> Result<Ordering, std::io::Error> {
  let mod_a = a.metadata()?.modified()?;
  let mod_b = b.metadata()?.modified()?;

  Result::Ok(mod_a.cmp(&mod_b))
}

pub async fn video_info_handler(Path((base_index, sub_dir)): Path<(u32, String)>) 
    -> (StatusCode, HeaderMap, Json<Vec<VideoEntity>>) {
  println!("{}", base_index);
  println!("{}", sub_dir);
  let mut sub_dir_param = String::from("/");
  sub_dir_param += &sub_dir;
  if sub_dir_param.ends_with("/") {
    sub_dir_param.truncate(sub_dir_param.len() - 1);
  }


  let mut conn = unsafe {
    POOL.unwrap().get_conn().unwrap()
  };

  let selected_video: Vec<VideoEntity> = conn.exec_map(
    "select id, video_file_name, cover_file_name from video_info where dir_path = :dir_path and base_index=:base_index", params! {
      "dir_path" => sub_dir_param,
      "base_index" => base_index,
    }, |(id, video_file_name, cover_file_name)| {VideoEntity{id, video_file_name, cover_file_name}}).unwrap();

  let mut header = HeaderMap::new();
  header.insert("Access-Control-Allow-Origin", "*".parse().unwrap());
  header.insert("content-type", "application/json; charset=utf-8".parse().unwrap());

  (StatusCode::OK, header, Json(selected_video))
}

#[derive(Serialize, Clone)]
pub struct VideoEntity {
  id: u32,
  #[serde(rename = "videoFileName")]
  video_file_name: String,
  #[serde(rename = "coverFileName")]
  cover_file_name: String
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

#[derive(Serialize)]
pub struct MountConfig {
  id: u32,
  #[serde(rename = "baseDir")]
  dir_path: String,
  #[serde(rename = "urlPrefix")]
  url_prefix: String,
  #[serde(rename = "apiVersion")]
  api_version: u32,
}