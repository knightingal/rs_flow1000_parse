use std::{cmp::Ordering, ffi::{c_char, CString}, fs::{self, DirEntry}};

use axum::{extract::Path, Json};
use hyper::{HeaderMap, StatusCode};
use mysql::{params, prelude::Queryable, Pool, Row};
use rusqlite::{named_params, Connection};
use serde_derive::Serialize;

use crate::{designation::parse_designation, get_mysql_connection, get_sqlite_connection, video_name_util::{parse_video_cover, VideoCover}};


pub static mut POOL: Option<&Pool> = None;
pub static mut SQLITE_CONN: Option<&Connection> = None;
pub static mut IS_LINUX: Option<&bool> = None;

#[repr(C)]
struct VideoMetaInfo {
  width: i32,
  height: i32,
  frame_rate: i32, 
  video_frame_count: i32,
  duratoin: i32,
}

#[link(name = "frame_decode")]
extern {
    fn frame_decode_with_param(file_url: *const c_char, dest_url: *const c_char) -> i32;
    fn video_meta_info(file_url: *const c_char) -> *mut VideoMetaInfo;
}

pub async fn video_detail(Path(id): Path<u32>) -> (StatusCode, Json<VideoEntity>) {
  let mut conn1 = get_mysql_connection();
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
      video_size: Option::Some(0),
    }}).unwrap();

  (StatusCode::OK, Json(selected_video.get(0).unwrap().clone()))
}


pub async fn generate_video_snapshot(Path(sub_dir): Path<String>) -> StatusCode {
  println!("{}", sub_dir);
  let path = std::path::Path::new(&sub_dir);
  let (video_name, image_name):(String, String) = if path.is_file() {
    let parent = path.parent().unwrap();
    let video_name = path.file_name().unwrap();
    let image_name = String::from(parent.to_str().unwrap()) + "/" + video_name.to_str().unwrap() + ".png";
    (sub_dir, image_name)
  } else {
    let ret = fs::read_dir(&sub_dir);
    if ret.is_err() {
      return StatusCode::NOT_FOUND;
    }
    let file_entry: Option<DirEntry> = ret.unwrap()
      .map(|res| res.unwrap())
      .find(|res| res.file_name().into_string().unwrap().ends_with(".mp4"));
    if file_entry.is_none() {
      return StatusCode::NOT_FOUND;
    }

    let video_name: String = file_entry.unwrap().path().into_os_string().into_string().unwrap();

    println!("{}", video_name);
    let mut img_name = video_name.clone();
    img_name.push_str(".png");
    (video_name, img_name)
  };

  unsafe {
    let video_name = CString::new(video_name).unwrap();
    let img_name = CString::new(image_name).unwrap();
    frame_decode_with_param( video_name.as_ptr(), img_name.as_ptr());
  }

  StatusCode::OK
}

pub async fn all_duplicate_cover() -> (StatusCode, Json<Vec<DuplicateCoverEntity>>) {
  let conn1 = get_sqlite_connection();

  let mut stmt = conn1.prepare(
    "select count, cover_file_name from(
      select count(vi.id) as count, vi.cover_file_name from video_info vi group by cover_file_name
    ) t where t.count > 1"
  ).unwrap();

   let mut duplicate_entity_list:Vec<DuplicateCoverEntity> = stmt.query_map(
     named_params! {},
    |row| { Ok(DuplicateCoverEntity{
      count: row.get_unwrap(0), 
      cover_file_name: row.get_unwrap(1),
      video_info_list: vec![]
    })}).unwrap().map(|it| it.unwrap()).collect();


  for duplicate_entity in &mut duplicate_entity_list {
    let mut stmt = conn1.prepare(
      "select id, video_file_name, cover_file_name, dir_path, base_index, designation_char, designation_num from video_info where cover_file_name=:cover_file_name "
    ).unwrap();

    let selected_video:Vec<VideoEntity> = stmt.query_map(
      named_params! {
        ":cover_file_name": &duplicate_entity.cover_file_name
      }, |row| {Ok(VideoEntity{
        id: row.get_unwrap(0), 
        video_file_name: row.get_unwrap(1), 
        cover_file_name: row.get_unwrap(2),
        dir_path: row.get_unwrap(3),
        base_index: row.get_unwrap(4), 
        designation_char: row.get_unwrap(5), 
        designation_num: row.get_unwrap(6),
        rate: Option::None,
        video_size: Option::Some(0),
      })}).unwrap().map(|it| it.unwrap()).collect();
    duplicate_entity.video_info_list = selected_video;
  }

  (StatusCode::OK, Json(duplicate_entity_list))
}


pub async fn all_duplicate_video() -> (StatusCode, Json<Vec<DuplicateEntity>>) {
  let conn1 = get_sqlite_connection();

  let mut stmt = conn1.prepare(
    "select 
      count, designation_char, designation_num 
    from (
        select count(vi.id) as count, count(DISTINCT vi.dir_path) as cd, count(DISTINCT vi.base_index) as cb, vi.designation_char , vi.designation_num  
        from video_info vi where vi.designation_char != 'MP' or vi.designation_num != '4' group by designation_char, designation_num) t 
    where t.count > 1  and t.cd > 1 ").unwrap();

   let mut duplicate_entity_list:Vec<DuplicateEntity> = stmt.query_map(
     named_params! {},
    |row| { Ok(DuplicateEntity{
      count: row.get_unwrap(0), 
      designation_char:row.get_unwrap(1), 
      designation_num: row.get_unwrap(2),
      video_info_list: vec![]
    })}).unwrap().map(|it| it.unwrap()).collect();


  for duplicate_entity in &mut duplicate_entity_list {
    let mut stmt = conn1.prepare(
      "select id, video_file_name, cover_file_name, dir_path, base_index from video_info where designation_char=:char and designation_num=:num "
    ).unwrap();

    let selected_video:Vec<VideoEntity> = stmt.query_map(
      named_params! {
        ":char" : &duplicate_entity.designation_char,
        ":num" : &duplicate_entity.designation_num,
      }, |row| {Ok(VideoEntity{
        id: row.get_unwrap(0), 
        video_file_name: row.get_unwrap(1), 
        cover_file_name: row.get_unwrap(2),
        designation_char: String::new(), 
        designation_num: String::new(),
        dir_path: row.get_unwrap(3),
        base_index: row.get_unwrap(4), 
        video_size: Option::Some(0),
        rate: Option::None
      })}).unwrap().map(|it| it.unwrap()).collect();
    duplicate_entity.video_info_list = selected_video;
  }

  (StatusCode::OK, Json(duplicate_entity_list))
}

pub async fn designation_search(Path(designation_ori): Path<String>) -> (StatusCode, Json<Vec<VideoEntity>>) {
  let designation = parse_designation(&designation_ori);
  let conn1 = get_sqlite_connection();

  let mut stmt = conn1.prepare("select 
    id, video_file_name, cover_file_name, dir_path, base_index 
  from 
    video_info 
  where 
    designation_char=:char and designation_num=:num").unwrap();

  let selected_video_iter = stmt.query_map(named_params! {
      ":char" : designation.char_final.unwrap(),
      ":num" : designation.num_final.unwrap(),
  }, |row| {
    Ok(VideoEntity{
          id: row.get_unwrap(0) ,
          video_file_name: row.get_unwrap(1), 
          cover_file_name: row.get_unwrap(2),
          designation_char: String::new(), 
          designation_num: String::new(),
          dir_path: row.get_unwrap(3),
          base_index: row.get_unwrap(4),
          rate: Option::Some(0),
          video_size: Option::Some(0),
    })
  }).unwrap().map(|it| it.unwrap());
  let selected_video:Vec<VideoEntity> = selected_video_iter.collect();
  let mut header = HeaderMap::new();
  header.insert("Access-Control-Allow-Origin", "*".parse().unwrap());
  header.insert("content-type", "application/json; charset=utf-8".parse().unwrap());
  (StatusCode::OK, Json(selected_video))
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

pub async fn parse_designation_handler(Path((base_index, sub_dir)): Path<(u32, String)>) 
    -> (StatusCode, HeaderMap, Json<Vec<VideoEntity>>) {
  println!("{}", base_index);
  println!("{}", sub_dir);
  let mut sub_dir_param = String::from("/");
  sub_dir_param += &sub_dir;
  if sub_dir_param.ends_with("/") {
    sub_dir_param.truncate(sub_dir_param.len() - 1);
  }

  let sqlite_conn = get_sqlite_connection();

  let mut stmt = sqlite_conn.prepare("select 
    id, video_file_name, cover_file_name 
  from 
    video_info 
  where 
    dir_path = :dir_path 
    and base_index=:base_index").unwrap();
  let selected_video: Vec<VideoEntity> = stmt.query_map(named_params! {
    ":dir_path": sub_dir_param, ":base_index": base_index
  }, |row| {
    let designation = parse_designation(&row.get_unwrap(1));
    return Ok(VideoEntity{
      id: row.get_unwrap(0),
      video_file_name: row.get_unwrap(1),
      cover_file_name: row.get_unwrap(2), 
      designation_char: designation.char_final.unwrap(), 
      designation_num: designation.num_final.unwrap(),
      dir_path: String::new(),
      base_index: 0,
      video_size: Option::Some(0),
      rate: Option::None,
    })
  }).unwrap().map(|it| it.unwrap()).collect();

  selected_video.iter().for_each(|video| {
    let mut stmt = sqlite_conn.prepare(
    "update 
      video_info 
    set 
      designation_char=:char, 
      designation_num=:num 
    where 
      id=:id"
    ).unwrap();
    let _ = stmt.execute(named_params! {
      "char": video.designation_char.clone(),
      "num": video.designation_num.clone(),
      "id": video.id,
    });
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
        video_size: Option::Some(0),
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

pub async fn sync_mysql2sqlite_mount_config() -> (StatusCode, HeaderMap, Json<Vec<MountConfig>>) {
  let mut conn = get_mysql_connection();
  let sqlite_conn = get_sqlite_connection();
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
    rusqlite::params![video_entity.id, video_entity.dir_path, video_entity.url_prefix, video_entity.api_version]).unwrap();
  });


  let mut header = HeaderMap::new();
  header.insert("Access-Control-Allow-Origin", "*".parse().unwrap());
  header.insert("content-type", "application/json; charset=utf-8".parse().unwrap());

  (StatusCode::OK, header, Json(mount_config))
}

pub async fn sync_mysql2sqlite_video_info() -> (StatusCode, HeaderMap, Json<Vec<VideoEntity>>) {
  let mut conn = get_mysql_connection();
  let sqlite_conn = get_sqlite_connection();

  let selected_video: Vec<VideoEntity> = conn.query_map(
    "select id, dir_path,base_index,rate, video_file_name, cover_file_name, designation_num, designation_char from video_info ", 
    |(id, dir_path, base_index, rate, video_file_name, cover_file_name, designation_num, designation_char)| {
      return VideoEntity{
        id, 
        video_file_name, 
        cover_file_name, 
        designation_char, 
        designation_num,
        dir_path,
        base_index,
        rate, 
        video_size: Option::Some(0),
      };
    }).unwrap();

  (&selected_video).into_iter().for_each(|video_entity| {
    sqlite_conn.execute("insert into video_info (
      id, dir_path, base_index,rate, video_file_name, cover_file_name, designation_num, designation_char
    ) values (
      ?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8
    )", rusqlite::params![
      video_entity.id, 
      video_entity.dir_path, 
      video_entity.base_index, 
      video_entity.rate, 
      video_entity.video_file_name, 
      video_entity.cover_file_name, 
      video_entity.designation_num, 
      video_entity.designation_char]).unwrap();
  });

  let mut header = HeaderMap::new();
  header.insert("Access-Control-Allow-Origin", "*".parse().unwrap());
  header.insert("content-type", "application/json; charset=utf-8".parse().unwrap());

  (StatusCode::OK, header, Json(selected_video))
}

pub async fn init_video_handler(Path((base_index, sub_dir)): Path<(u32, String)>) 
    -> (StatusCode, HeaderMap, Json<Vec<VideoCover>>) {
  println!("{}", base_index);
  println!("{}", sub_dir);
  let mut sub_dir_param = String::from("/");
  sub_dir_param += &sub_dir;

  let sqlite_conn = unsafe {
    SQLITE_CONN.unwrap()
  };

  let mut stmt = sqlite_conn.prepare("select dir_path from mp4_base_dir where id = :id").unwrap();
  let mut dir_path: String = stmt.query_row(named_params! {":id": base_index}, |row| {
    Ok(row.get_unwrap(0))
  }).unwrap();

  dir_path += &sub_dir_param;

  let file_names: Vec<(String, u64)> = parse_dir_path(&dir_path).unwrap();
  let video_cover_list = parse_video_cover(&file_names);

  for video_cover_entry in video_cover_list.iter() {
    let designation = parse_designation(&video_cover_entry.video_file_name);
    let exist = check_exist_by_video_file_name(&sub_dir_param, base_index, &video_cover_entry.video_file_name);

    if !exist {
      let _ = sqlite_conn.execute("insert into video_info(
        dir_path, base_index, video_file_name, cover_file_name, designation_char, designation_num, video_size
      ) values (
        :dir_path, :base_index, :video_file_name, :cover_file_name, :designation_char, :designation_num, video_size
      )", named_params! {
        ":dir_path": sub_dir_param, 
        ":base_index": base_index, 
        ":video_file_name": video_cover_entry.video_file_name, 
        ":cover_file_name": video_cover_entry.cover_file_name,
        ":designation_char": designation.char_final, 
        ":designation_num": designation.num_final,
        "video_size": video_cover_entry.video_size,
      });
    } else {
      let _ = sqlite_conn.execute("update video_info set 
        cover_file_name=:cover_file_name, 
        designation_char=:designation_char, 
        designation_num=:designation_num, 
        video_size=:video_size
      where
        dir_path=:dir_path and base_index=:base_index and video_file_name=:video_file_name
      ", named_params! {
        ":dir_path": sub_dir_param, 
        ":base_index": base_index, 
        ":video_file_name": video_cover_entry.video_file_name, 
        ":cover_file_name": video_cover_entry.cover_file_name,
        ":designation_char": designation.char_final, 
        ":designation_num": designation.num_final,
        "video_size": video_cover_entry.video_size,
      });

    }
  }

  println!("{:?}", video_cover_list);

  let mut header = HeaderMap::new();
  header.insert("Access-Control-Allow-Origin", "*".parse().unwrap());
  header.insert("content-type", "application/json; charset=utf-8".parse().unwrap());

  (StatusCode::OK, header, Json(video_cover_list))
}

fn check_exist_by_video_file_name(dir_path: &String, base_index: u32, video_file_name: &String) -> bool {
  let sqlite_conn = unsafe {
    SQLITE_CONN.unwrap()
  };
  let mut stmt = sqlite_conn.prepare(
  "select 
    count(id) 
  from 
    video_info 
  where 
    dir_path=:dir_path 
    and base_index=:base_index 
    and video_file_name=:video_file_name").unwrap();
  let count: u32 = stmt.query_row(named_params! {
    ":video_file_name": video_file_name, 
    ":base_index": base_index, 
    ":dir_path": dir_path, 
  }, |row| {
    Ok(row.get_unwrap(0))
  }).unwrap();

  count != 0
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
  #[serde(rename = "videoSize")]
  pub video_size: Option<u64>,
}


#[derive(Serialize, Clone)]
pub struct DuplicateCoverEntity {
  count: u32,
  #[serde(rename = "coverFileName")]
  cover_file_name: String,
  #[serde(rename = "videoInfo")]
  video_info_list: Vec<VideoEntity>,
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
pub struct MountConfig {
  pub id: u32,
  #[serde(rename = "baseDir")]
  pub dir_path: String,
  #[serde(rename = "urlPrefix")]
  pub url_prefix: String,
  #[serde(rename = "apiVersion")]
  pub api_version: u32,
}