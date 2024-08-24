use std::{cmp::Ordering, fs::{self, DirEntry}};

use axum::{extract::Path, Json};
use hyper::{HeaderMap, StatusCode};
use mysql::{params, prelude::Queryable, Pool, Row};
use rusqlite::{named_params, Connection};
use serde_derive::Serialize;

use crate::{designation::parse_designation, get_mysql_connection, get_sqlite_connection, video_name_util::{parse_video_cover, VideoCover}};


pub static mut POOL: Option<&Pool> = None;
pub static mut SQLITE_CONN: Option<&Connection> = None;
pub static mut IS_LINUX: Option<&bool> = None;

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
      video_size:0,
    }}).unwrap();

  (StatusCode::OK, Json(selected_video.get(0).unwrap().clone()))
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
        video_size:0,
      }
    )
  });

  let mut header = HeaderMap::new();
  header.insert("Access-Control-Allow-Origin", "*".parse().unwrap());
  header.insert("content-type", "application/json; charset=utf-8".parse().unwrap());

  (StatusCode::OK, header, Json(result.unwrap().clone()))
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
        video_size:0,
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
        video_size:0,
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
          video_size:0,
    })
  }).unwrap().map(|it| it.unwrap());
  let selected_video:Vec<VideoEntity> = selected_video_iter.collect();
  let mut header = HeaderMap::new();
  header.insert("Access-Control-Allow-Origin", "*".parse().unwrap());
  header.insert("content-type", "application/json; charset=utf-8".parse().unwrap());
  (StatusCode::OK, Json(selected_video))
}

pub async fn mount_config_handler()  
    -> (StatusCode, HeaderMap, Json<Vec<MountConfig>>) {


  let sqlite_conn = get_sqlite_connection();

  let mut sql = String::from("select id, ");
  unsafe {
    if *IS_LINUX.unwrap() {
      sql += "dir_path ";
    } else {
      sql += "win_dir_path ";
    }
  }
  sql += " , url_prefix, api_version from mp4_base_dir ";

  let mut stmt = sqlite_conn.prepare(sql.as_str()).unwrap();
  let mount_config_iter = stmt.query_map(named_params! {}, |row| {
    Ok(MountConfig{
        id: row.get_unwrap(0),
        dir_path: row.get_unwrap(1),
        url_prefix: row.get_unwrap(2),
        api_version: row.get_unwrap(3),
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

  let file_names = parse_dir_path(&dir_path).unwrap();

  let mut header = HeaderMap::new();
  header.insert("Access-Control-Allow-Origin", "*".parse().unwrap());
  header.insert("content-type", "application/json; charset=utf-8".parse().unwrap());

  (StatusCode::OK, header, Json(file_names))
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
  let mut sub_dir_param = String::from("/");
  sub_dir_param += &sub_dir;
  if sub_dir_param.ends_with("/") {
    sub_dir_param.truncate(sub_dir_param.len() - 1);
  }

  let sqlite_conn = get_sqlite_connection();

  let mut stmt = sqlite_conn.prepare("select id, video_file_name, cover_file_name, rate from video_info where dir_path = :dir_path and base_index=:base_index").unwrap();
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
      video_size:0,
      rate: row.get_unwrap(3)
    })
  }).unwrap().map(|it| it.unwrap());

  let selected_video:Vec<VideoEntity> = selected_video_iter.collect();

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
      video_size:0,
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
        video_size:0,
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
        video_size:0
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

  let file_names = parse_dir_path(&dir_path).unwrap();
  let video_cover_list = parse_video_cover(&file_names);

  for video_cover_entry in video_cover_list.iter() {
    let designation = parse_designation(&video_cover_entry.video_file_name);
    let exist = check_exist_by_video_file_name(&sub_dir_param, base_index, &video_cover_entry.video_file_name);

    if !exist {
      let _ = sqlite_conn.execute("insert into video_info(
        dir_path, base_index, video_file_name, cover_file_name, designation_char, designation_num
      ) values (
        :dir_path, :base_index, :video_file_name, :cover_file_name, :designation_char, :designation_num
      )", named_params! {
        ":dir_path": sub_dir_param, 
        ":base_index": base_index, 
        ":video_file_name": video_cover_entry.video_file_name, 
        ":cover_file_name": video_cover_entry.cover_file_name,
        ":designation_char": designation.char_final, 
        ":designation_num": designation.num_final,
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
  id: u32,
  #[serde(rename = "videoFileName")]
  video_file_name: String,
  #[serde(rename = "coverFileName")]
  cover_file_name: String,
  #[serde(rename = "designationChar")]
  designation_char: String,
  #[serde(rename = "designationNum")]
  designation_num: String,
  #[serde(rename = "dirPath")]
  dir_path: String,
  #[serde(rename = "baseIndex")]
  base_index: u32,
  #[serde(rename = "rate")]
  rate: Option<u32>,
  #[serde(rename = "videoSize")]
  video_size: u64,
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
  id: u32,
  #[serde(rename = "baseDir")]
  dir_path: String,
  #[serde(rename = "urlPrefix")]
  url_prefix: String,
  #[serde(rename = "apiVersion")]
  api_version: u32,
}
