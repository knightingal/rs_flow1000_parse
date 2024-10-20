
use std::{cmp::Ordering, fs::{self, DirEntry}};

use axum::{extract::Path, Json};
use hyper::{HeaderMap, StatusCode};
use rusqlite::{named_params, Connection};

use crate::{entity::*, handles::{IS_LINUX, SQLITE_CONN}};

fn get_sqlite_connection() -> &'static Connection {
  let conn: &Connection = unsafe {
    SQLITE_CONN.unwrap()
  };
  return conn;
}

pub async fn video_info_handler(Path((base_index, sub_dir)): Path<(u32, String)>) 
    -> (StatusCode, HeaderMap, Json<Vec<VideoEntity>>) {
  let mut sub_dir_param = String::from("/");
  sub_dir_param += &sub_dir;
  if sub_dir_param.ends_with("/") {
    sub_dir_param.truncate(sub_dir_param.len() - 1);
  }

  let sqlite_conn = get_sqlite_connection();

  let mut stmt = sqlite_conn.prepare("select id, video_file_name, cover_file_name, rate, video_size from video_info where dir_path = :dir_path and base_index=:base_index").unwrap();
  let selected_video_iter = stmt.query_map(named_params! {":dir_path": sub_dir_param.as_str(),":base_index": base_index}, |row| {
    Ok(VideoEntity{
      // id: row.get(0)?,
      id: row.get_unwrap(0),
      video_file_name: row.get_unwrap(1),
      cover_file_name: row.get_unwrap(2),
      designation_char: String::new(), 
      designation_num: String::new(),
      dir_path: String::new(),
      base_index: 0,
      video_size: row.get_unwrap(4),
      rate: row.get_unwrap(3),
      height:0,
      width: 0,
      frame_rate: 0,
      video_frame_count: 0,
      duration: 0,
    })
  }).unwrap().map(|it| it.unwrap());

  let selected_video:Vec<VideoEntity> = selected_video_iter.collect();

  let mut header = HeaderMap::new();
  header.insert("Access-Control-Allow-Origin", "*".parse().unwrap());
  header.insert("content-type", "application/json; charset=utf-8".parse().unwrap());

  (StatusCode::OK, header, Json(selected_video))
}

pub async fn mount_config_handler()  
    -> (StatusCode, HeaderMap, Json<Vec<MountConfig>>) {


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
  sql += " , url_prefix, api_version from mp4_base_dir ";

  let mut stmt = sqlite_conn.prepare(sql.as_str()).unwrap();
  let mount_config_iter = stmt.query_map(named_params! {}, |row| {
    Ok(MountConfig{
        id: row.get_unwrap("id"),
        dir_path: row.get_unwrap(dir_path_name),
        url_prefix: row.get_unwrap("url_prefix"),
        api_version: row.get_unwrap("api_version"),
    })
  }).unwrap().map(|it| it.unwrap());

  let mount_config_list:Vec<MountConfig> = mount_config_iter.collect();

  let mut header = HeaderMap::new();
  header.insert("Access-Control-Allow-Origin", "*".parse().unwrap());
  header.insert("content-type", "application/json; charset=utf-8".parse().unwrap());

  (StatusCode::OK, header, Json(mount_config_list))
}

pub async fn mp4_dir_handler1(Path(base_index): Path<u32>) 
    -> (StatusCode, HeaderMap, Json<Vec<String>>) {
  println!("{}", base_index);

  let sqlite_conn = get_sqlite_connection();

  let mut sql = String::from("select ");
  unsafe {
    if *IS_LINUX.unwrap() {
      sql += "dir_path ";
    } else {
      sql += "win_dir_path ";
    }
  }
  sql += " from mp4_base_dir where id = :id";

  let mut stmt = sqlite_conn.prepare(sql.as_str()).unwrap();
  let dir_path: String = stmt.query_row(named_params! {":id": base_index}, |row| {
    Ok(row.get_unwrap(0))
  }).unwrap();

  let file_names:Vec<(String, u64)> = parse_dir_path(&dir_path).unwrap();

  let mut header = HeaderMap::new();
  header.insert("Access-Control-Allow-Origin", "*".parse().unwrap());
  header.insert("content-type", "application/json; charset=utf-8".parse().unwrap());

  (StatusCode::OK, header, Json(file_names.into_iter().map(|f| f.0).collect()))
}

pub async fn mp4_dir_handler(Path((base_index, sub_dir)): Path<(u32, String)>) 
    -> (StatusCode, HeaderMap, Json<Vec<String>>) {
  let mut sub_dir_param = String::from("/");
  sub_dir_param += &sub_dir;

  let sqlite_conn = get_sqlite_connection();

  let mut sql = String::from("select ");
  unsafe {
    if *IS_LINUX.unwrap() {
      sql += "dir_path ";
    } else {
      sql += "win_dir_path ";
    }
  }
  sql += " from mp4_base_dir where id = :id";

  let mut stmt = sqlite_conn.prepare(sql.as_str()).unwrap();
  let mut dir_path: String = stmt.query_row(named_params! {":id": base_index}, |row| {
    Ok(row.get_unwrap(0))
  }).unwrap();

  dir_path += "/";
  dir_path += &sub_dir;

  let file_names = parse_dir_path(&dir_path);
  let mut header = HeaderMap::new();
  header.insert("Access-Control-Allow-Origin", "*".parse().unwrap());
  header.insert("content-type", "application/json; charset=utf-8".parse().unwrap());

  (StatusCode::OK, header, Json(file_names.unwrap().into_iter().map(|f| f.0).collect()))
}

pub async fn video_rate(Path((id, rate)): Path<(u32, u32)>) -> (StatusCode, HeaderMap, Json<VideoEntity>) {
  let sqlite_conn = get_sqlite_connection();

  sqlite_conn.execute("update video_info set rate=?1 where id=?2", rusqlite::params![rate, id]).unwrap();
  let result: Result<VideoEntity, _> = sqlite_conn.query_row("select id, video_file_name, cover_file_name, rate from video_info where id = :id ", named_params! {
      ":id" : id,
  }, |row| {
    Result::Ok(
      VideoEntity{
        id: row.get_unwrap(0),
        video_file_name: row.get_unwrap(1),
        cover_file_name: row.get_unwrap(2),
        designation_char: String::new(), 
        designation_num: String::new(),
        dir_path: String::new(),
        base_index: 0,
        rate: row.get_unwrap(3),
        video_size: Option::Some(0),
        height:0,
        width: 0,
        frame_rate: 0,
        video_frame_count: 0,
        duration: 0,
      }
    )
  });

  let mut header = HeaderMap::new();
  header.insert("Access-Control-Allow-Origin", "*".parse().unwrap());
  header.insert("content-type", "application/json; charset=utf-8".parse().unwrap());

  (StatusCode::OK, header, Json(result.unwrap().clone()))
}

pub async fn add_tag(Path(tag_name): Path<String>) -> (StatusCode, HeaderMap, Json<TagEntity>) {

  let sqlite_conn = get_sqlite_connection();

  let count: usize = sqlite_conn.query_row("select count(id) from tag where tag = :tag", named_params! {":tag": tag_name}, |row| {
    Result::Ok(row.get_unwrap(0))
  }).unwrap();

  if count == 0 {
    let _ = sqlite_conn.execute("insert into tag (tag) values (:tag)", named_params! {":tag": tag_name});
  }


  let tag_entity: TagEntity = sqlite_conn.query_row("select id, tag from tag where tag=:tag", named_params! {":tag": tag_name}, |row| {
    Result::Ok(
      TagEntity {id: row.get_unwrap("id"), tag: row.get_unwrap("tag")}
    )
  }).unwrap();
  

  let mut header = HeaderMap::new();
  header.insert("Access-Control-Allow-Origin", "*".parse().unwrap());

  (StatusCode::OK, header, Json(tag_entity))
}

fn parse_dir_path(dir_path: &String) -> Result<Vec<(String, u64)>, std::io::Error> {
  let mut file_entry_list: Vec<DirEntry> = fs::read_dir(dir_path)?
    .map(|res| res.unwrap())
    .filter(|res| !res.file_name().into_string().unwrap().ends_with(".torrent")).collect();
  file_entry_list.sort_by(|a, b| comp_path(&b, &a).unwrap());

  let file_names:Vec<(String, u64)> = file_entry_list.into_iter().map(|res| 
      (res
        .file_name()
        .into_string()
        .unwrap(), res.metadata().unwrap().len())
  ).collect();

  return Result::Ok(file_names);
}

fn comp_path(a: &DirEntry, b: &DirEntry) -> Result<Ordering, std::io::Error> {
  let mod_a = a.metadata()?.modified()?;
  let mod_b = b.metadata()?.modified()?;

  Result::Ok(mod_a.cmp(&mod_b))
}
