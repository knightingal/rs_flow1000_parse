use std::{cmp::Ordering, fs::{self, DirEntry}};

use axum::{extract::{Path, State}, Json};
use hyper::{HeaderMap, StatusCode};
use mysql::{params, prelude::Queryable, Pool, Row};
use rusqlite::Connection;
use serde_derive::Serialize;

use crate::{designation::parse_designation, get_sqlite_connection};


pub static mut POOL: Option<&Pool>= None;
pub static mut SQLITE_CONN: Option<&Connection>= None;

pub async fn video_detail(State(pool): State<Pool>, Path(id): Path<u32>) -> (StatusCode, Json<VideoEntity>) {
  let mut conn1 = pool.get_conn().unwrap();
  let selected_video = conn1.exec_map(
    "select id, video_file_name, cover_file_name from video_info where id = :id ", params! {
      "id" => id,
    }, |(id, video_file_name, cover_file_name)| {VideoEntity{
      id, 
      video_file_name, 
      cover_file_name,
      designation_char: String::new(), 
      designation_num: String::new(),
      dir_path: String::new(),
      base_index: 0,
      rate: Option::Some(0),
    }}).unwrap();

  (StatusCode::OK, Json(selected_video.get(0).unwrap().clone()))
}

pub async fn video_rate(State(pool): State<Pool>, Path((id, rate)): Path<(u32, u32)>) -> (StatusCode, HeaderMap, Json<VideoEntity>) {
  let mut conn1 = pool.get_conn().unwrap();

  let sqlite_conn = get_sqlite_connection();
  let _:Vec<Row> = conn1.exec("update video_info set rate=:rate where id=:id", params! {
    "rate" => rate,
    "id" => id
  }).unwrap();

  sqlite_conn.execute("update video_info set rate=?1 where id=?2", rusqlite::params![rate, id]).unwrap();

  let selected_video = conn1.exec_map(
    "select id, video_file_name, cover_file_name, rate from video_info where id = :id ", params! {
      "id" => id,
    }, |(id, video_file_name, cover_file_name, rate)| {VideoEntity{
      id, 
      video_file_name, 
      cover_file_name,
      designation_char: String::new(), 
      designation_num: String::new(),
      dir_path: String::new(),
      base_index: 0,
      rate,
    }}).unwrap();

  let mut header = HeaderMap::new();
  header.insert("Access-Control-Allow-Origin", "*".parse().unwrap());
  header.insert("content-type", "application/json; charset=utf-8".parse().unwrap());

  (StatusCode::OK, header, Json(selected_video.get(0).unwrap().clone()))
}

pub async fn all_duplicate_video(State(pool): State<Pool>) -> (StatusCode, Json<Vec<DuplicateEntity>>) {
  let mut conn1 = pool.get_conn().unwrap();
  let mut duplicate_entity_list:Vec<DuplicateEntity> = conn1.query_map(
    "select 
      count, designation_char, designation_num 
    from (
        select count(vi.id) as count, count(DISTINCT vi.dir_path) as cd, count(DISTINCT vi.base_index) as cb, vi.designation_char , vi.designation_num  
        from video_info vi where vi.designation_char != 'MP' or vi.designation_num != '4' group by designation_char, designation_num) t 
    where t.count > 1  and t.cd > 1 ", 
    |(count, designation_char, designation_num)| {DuplicateEntity{
      count, 
      designation_char, 
      designation_num,
      video_info_list: vec![]
    }}).unwrap();
  for duplicate_entity in &mut duplicate_entity_list {
    let selected_video:Vec<VideoEntity> = conn1.exec_map(
      "select id, video_file_name, cover_file_name, dir_path, base_index from video_info where designation_char=:char and designation_num=:num ", params! {
        "char" => &duplicate_entity.designation_char,
        "num" => &duplicate_entity.designation_num,
      }, |(id, video_file_name, cover_file_name, dir_path, base_index)| {VideoEntity{
        id, 
        video_file_name, 
        cover_file_name,
        designation_char: String::new(), 
        designation_num: String::new(),
        dir_path,
        base_index, 
        rate: Option::None
      }}).unwrap();
    duplicate_entity.video_info_list = selected_video;
  }


  (StatusCode::OK, Json(duplicate_entity_list))
}

pub async fn designation_search(State(pool): State<Pool>, Path(designation_ori): Path<String>) -> (StatusCode, Json<Vec<VideoEntity>>) {
  let designation = parse_designation(&designation_ori);
  let mut conn1 = pool.get_conn().unwrap();
  let selected_video:Vec<VideoEntity> = conn1.exec_map(
    "select id, video_file_name, cover_file_name, dir_path, base_index from video_info where designation_char=:char and designation_num=:num ", params! {
      "char" => designation.char_final.unwrap(),
      "num" => designation.num_final.unwrap(),
    }, |(id, video_file_name, cover_file_name, dir_path, base_index)| {VideoEntity{
      id, 
      video_file_name, 
      cover_file_name,
      designation_char: String::new(), 
      designation_num: String::new(),
      dir_path,
      base_index,
      rate: Option::Some(0),
    }}).unwrap();
  (StatusCode::OK, Json(selected_video))
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
    "select id, video_file_name, cover_file_name, rate from video_info where dir_path = :dir_path and base_index=:base_index", params! {
      "dir_path" => sub_dir_param,
      "base_index" => base_index,
    }, |(id, video_file_name, cover_file_name, rate)| {VideoEntity{
      id, 
      video_file_name, 
      cover_file_name,
      designation_char: String::new(), 
      designation_num: String::new(),
      dir_path: String::new(),
      base_index: 0,
      rate
    }}).unwrap();

  let mut header = HeaderMap::new();
  header.insert("Access-Control-Allow-Origin", "*".parse().unwrap());
  header.insert("content-type", "application/json; charset=utf-8".parse().unwrap());

  (StatusCode::OK, header, Json(selected_video))
}

pub async fn parse_designation_handler(Path((base_index, sub_dir)): Path<(u32, String)>) 
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
    }, |(id, video_file_name, cover_file_name)| {
      let designation = parse_designation(&video_file_name);

      return VideoEntity{
        id, 
        video_file_name, 
        cover_file_name, 
        designation_char: designation.char_final.unwrap(), 
        designation_num: designation.num_final.unwrap(),
        dir_path: String::new(),
        base_index: 0,
        rate: Option::None
      };
    }).unwrap();

  selected_video.iter().for_each(|video| {
    let _:Vec<Row> = conn.exec("update video_info set designation_char=:char, designation_num=:num where id=:id", params! {
      "char" => video.designation_char.clone(),
      "num" => video.designation_num.clone(),
      "id" => video.id
    }).unwrap();

  });

  let mut header = HeaderMap::new();
  header.insert("Access-Control-Allow-Origin", "*".parse().unwrap());
  header.insert("content-type", "application/json; charset=utf-8".parse().unwrap());

  (StatusCode::OK, header, Json(selected_video))
}

pub async fn parse_designation_all_handler() 
    -> (StatusCode, HeaderMap, Json<Vec<VideoEntity>>) {

  let mut conn = unsafe {
    POOL.unwrap().get_conn().unwrap()
  };

  let selected_video: Vec<VideoEntity> = conn.query_map(
    "select id, video_file_name, cover_file_name from video_info ", |(id, video_file_name, cover_file_name)| {
      let designation = parse_designation(&video_file_name);

      return VideoEntity{
        id, 
        video_file_name, 
        cover_file_name, 
        designation_char: designation.char_final.unwrap(), 
        designation_num: designation.num_final.unwrap(),
        dir_path: String::new(),
        base_index: 0,
        rate: Option::None
      };
    }).unwrap();

  selected_video.iter().for_each(|video| {
    let _:Vec<Row> = conn.exec("update video_info set designation_char=:char, designation_num=:num where id=:id", params! {
      "char" => video.designation_char.clone(),
      "num" => video.designation_num.clone(),
      "id" => video.id
    }).unwrap();

  });

  let mut header = HeaderMap::new();
  header.insert("Access-Control-Allow-Origin", "*".parse().unwrap());
  header.insert("content-type", "application/json; charset=utf-8".parse().unwrap());

  (StatusCode::OK, header, Json(selected_video))
}

#[derive(Serialize, Clone)]
pub struct VideoEntity {
  pub id: u32,
  #[serde(rename = "videoFileName")]
  pub video_file_name: String,
  #[serde(rename = "coverFileName")]
  pub cover_file_name: String,
  #[serde(rename = "designationChar")]
  pub designation_char: String,
  #[serde(rename = "designationNum")]
  pub designation_num: String,
  #[serde(rename = "dirPath")]
  pub dir_path: String,
  #[serde(rename = "baseIndex")]
  pub base_index: u32,
  #[serde(rename = "rate")]
  pub rate: Option<u32>,
}

#[derive(Serialize, Clone)]
pub struct DuplicateEntity {
  count: u32,
  #[serde(rename = "designationChar")]
  designation_char: String,
  #[serde(rename = "designationNum")]
  designation_num: String,

  #[serde(rename = "videoInfo")]
  video_info_list: Vec<VideoEntity>,
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
  pub id: u32,
  #[serde(rename = "baseDir")]
  pub dir_path: String,
  #[serde(rename = "urlPrefix")]
  pub url_prefix: String,
  #[serde(rename = "apiVersion")]
  pub api_version: u32,
}