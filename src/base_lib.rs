use std::{cmp::Ordering, env, fs::{self, DirEntry}};

use rusqlite::{Connection, named_params};
use sysinfo::System;

use crate::{entity::{MountConfig, VideoEntity}, video_name_util::parse_video_meta_info};

#[cfg(reallink)]
#[link(name = "cfb_decode")]
extern "C" {
  #[allow(dead_code)]
  fn init_inner_key_expansion(key: *const u8);
}

pub static mut IS_LINUX: Option<&bool> = None;

pub fn hex_to_byte_array(hex: String) -> [u8; 32] {
  let mut byte_array: [u8; 32] = [0; 32];
  for i in 0..32 {
    let byte_str = &hex[i * 2..i * 2 + 2];
    byte_array[i] = u8::from_str_radix(byte_str, 16).unwrap();
  }
  byte_array
}

pub fn linux_init() {
  let is_linux = Box::new(
    System::name().unwrap().contains("Linux")
      || System::name().unwrap() == "Deepin"
      || System::name().unwrap().contains("openSUSE"),
  );
  unsafe {
    IS_LINUX = Some(Box::leak(is_linux));
  }
}

pub fn get_sqlite_connection() -> Connection {
  let db_path_env = env::var("DB_PATH")
    .unwrap_or_else(|_| String::from("/home/knightingal/source/keys/mp41000.db"));
  let conn = Connection::open(db_path_env).unwrap();
  return conn;
}

pub fn query_mount_configs() -> Vec<MountConfig> {

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
  let mount_config_iter = stmt
    .query_map(named_params! {}, |row| {
      Ok(MountConfig {
        id: row.get_unwrap("id"),
        dir_path: row.get_unwrap(dir_path_name),
        url_prefix: row.get_unwrap("url_prefix"),
        api_version: row.get_unwrap("api_version"),
      })
    })
    .unwrap()
    .map(|it| it.unwrap());

  let mount_config_list: Vec<MountConfig> = mount_config_iter.collect();
  return mount_config_list;
}


pub fn video_entity_to_file_path(video_entity: &VideoEntity, mount_configs: &Vec<MountConfig>) -> (String, String, String) {
  let mount_config = mount_configs.iter().find(|it| it.id == video_entity.base_index).unwrap();
  let mut video_path = mount_config.dir_path.clone();
  video_path.push_str(&video_entity.dir_path);
  video_path.push_str("/");
  video_path.push_str(&video_entity.video_file_name);

  let mut cover_path = mount_config.dir_path.clone();
  cover_path.push_str(&video_entity.dir_path);
  cover_path.push('/');
  cover_path.push_str(&video_entity.cover_file_name);

  let mut dir_path = mount_config.dir_path.clone();
  dir_path.push_str(&video_entity.dir_path);

  (video_path, cover_path, dir_path)
} 


pub fn parse_dir_path(dir_path: &String) -> Result<Vec<(String, u64)>, std::io::Error> {
  let mut file_entry_list: Vec<DirEntry> = fs::read_dir(dir_path)?
    .map(|res| res.unwrap())
    .filter(|res| !res.file_name().into_string().unwrap().ends_with(".torrent"))
    .collect();
  file_entry_list.sort_by(|a, b| comp_path(&b, &a).unwrap());

  let file_names: Vec<(String, u64)> = file_entry_list
    .into_iter()
    .map(|res| {
      (
        res.file_name().into_string().unwrap(),
        res.metadata().unwrap().len(),
      )
    })
    .collect();

  return Result::Ok(file_names);
}

fn comp_path(a: &DirEntry, b: &DirEntry) -> Result<Ordering, std::io::Error> {
  let mod_a = a.metadata()?.modified()?;
  let mod_b = b.metadata()?.modified()?;

  Result::Ok(mod_a.cmp(&mod_b))
}


pub fn check_exist_by_video_file_name(
  dir_path: &String,
  base_index: u32,
  video_file_name: &String,
) -> bool {
  let sqlite_conn = get_sqlite_connection();
  let mut stmt = sqlite_conn
    .prepare(
      "select 
    count(id) 
  from 
    video_info 
  where 
    dir_path=:dir_path 
    and base_index=:base_index 
    and video_file_name=:video_file_name",
    )
    .unwrap();
  let count: u32 = stmt
    .query_row(
      named_params! {
        ":video_file_name": video_file_name,
        ":base_index": base_index,
        ":dir_path": dir_path,
      },
      |row| Ok(row.get_unwrap(0)),
    )
    .unwrap();

  count != 0
}

pub fn init_key() {

  let cfb_key = env::var("CFB_KEY");

  let pwd: [u8; 32] = match cfb_key {
      Ok(cfb_key) => hex_to_byte_array(cfb_key),
      Err(_) => {
        tracing::warn!("CFB_KEY not set, use hard coded key");
        let key = "passwordpasswordpasswordpassword"; // 32 bytes key
        key.as_bytes().try_into().unwrap()
      },
  };

  unsafe {
    init_inner_key_expansion(pwd.as_ptr());
  }

}

pub fn video_file_path_by_id(id: u32) -> Vec<(u32, String, String)>{

  let mount_config_list = query_mount_configs();

  println!("call query video_file_name");
  let sqlite_conn = get_sqlite_connection();

  let mut stmt = sqlite_conn
    .prepare(
      "select 
        id, video_file_name, base_index, dir_path, cover_file_name
      from 
        video_info 
      where 
        id = :id",
    )
    .unwrap();
  let file_names: Vec<(u32, String, String)> = stmt
    .query_map(named_params! {":id": id}, |row| {
      let video_file_name: String = row.get_unwrap("video_file_name");
      let cover_file_name: String = row.get_unwrap("cover_file_name");
      let dir_path: String = row.get_unwrap("dir_path");
      let base_index: u32 = row.get_unwrap("base_index");
      let id: u32 = row.get_unwrap("id");
      println!("get file_name:{}, {}", video_file_name, cover_file_name);

      let (video_full_name, cover_full_name, _) = video_entity_to_file_path(&VideoEntity::new_by_file_name(
        id, video_file_name, cover_file_name, dir_path, base_index
      ), &mount_config_list);
      println!("{}", cover_full_name);

      Result::Ok((id, video_full_name, cover_full_name))
    })
    .unwrap()
    .map(|it| it.unwrap())
    .collect();
  return file_names;
}

pub fn parse_and_update_meta_info_by_id(id: u32, video_file_name: String, cover_file_name: String) {
  let sqlite_conn: Connection = get_sqlite_connection();
  let mut stmt: rusqlite::Statement<'_> = sqlite_conn
    .prepare(
      "update 
    video_info 
  set 
    video_size = :video_size,
    cover_size = :cover_size,
    width = :width,
    height = :height,
    frame_rate = :frame_rate,
    video_frame_count=:video_frame_count,
    duration=:duration 
  where 
    id=:id",
    )
    .unwrap();

  let path = std::path::Path::new(&video_file_name);
  let exist = path.exists();
  if !exist {
    return;
  }
  let video_file_size = path.metadata().map_or_else(|_| 0, |m| m.len());

  let path = std::path::Path::new(&cover_file_name);
  let exist = path.exists();
  if !exist {
    return;
  }
  let cover_file_size = path.metadata().map_or_else(|_| 0, |m| m.len());

  println!("parse file:{}", video_file_name);

  let meta_info = parse_video_meta_info(&video_file_name);
  let _ = stmt.execute(named_params! {
    ":width": meta_info.width,
    ":height": meta_info.height,
    ":frame_rate": meta_info.frame_rate,
    ":video_size": video_file_size,
    ":cover_size": cover_file_size,
    ":duration":meta_info.duratoin,
    ":video_frame_count": meta_info.video_frame_count,
    ":id": id
  });
}