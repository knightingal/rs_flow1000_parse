use std::{cmp::Ordering, env, fs::{self, DirEntry}};

use rusqlite::{Connection, named_params};
use sysinfo::System;

use crate::entity::{MountConfig, VideoEntity};

pub static mut IS_LINUX: Option<&bool> = None;

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