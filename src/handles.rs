use core::slice;
use std::{
  cmp::Ordering,
  collections::HashMap,
  ffi::{c_char, c_void, CString},
  fs::{self, DirBuilder, DirEntry},
  thread, usize,
};

use axum::{
  body::{Body, Bytes},
  extract::{Path, Query},
  response::Response,
  Error, Json,
};
use hyper::{
  header::{ACCESS_CONTROL_ALLOW_ORIGIN, CONTENT_LENGTH, CONTENT_TYPE},
  HeaderMap, StatusCode,
};
use rusqlite::{named_params, params_from_iter, Connection};

use crate::{
  designation::parse_designation,
  entity::*,
  get_sqlite_connection,
  video_name_util::{parse_video_cover, parse_video_meta_info, VideoCover, VideoMetaInfo},
};

pub static mut IS_LINUX: Option<&bool> = None;

#[repr(C)]
pub struct SnapshotSt {
  pub buff: *const u8,
  pub buff_len: i32,
}

#[cfg(reallink)]
#[link(name = "frame_decode")]
extern "C" {
  fn frame_decode_with_param(file_url: *const c_char, dest_url: *const c_char) -> i32;
  fn snapshot_video(file_url: *const c_char, snap_time: u64) -> SnapshotSt;
}

#[cfg(reallink)]
#[link(name = "avformat")]
extern "C" {}

#[cfg(reallink)]
#[link(name = "swscale")]
extern "C" {}

#[cfg(mocklink)]
fn frame_decode_with_param(_: *const c_char, _: *const c_char) -> i32 {
  return 0;
}

pub async fn video_detail(Path(id): Path<u32>) -> (StatusCode, Json<Option<VideoEntity>>) {
  let conn = get_sqlite_connection();
  let video_entity = conn
    .query_row(
      "select id, video_file_name, cover_file_name, base_index, dir_path from video_info where id = :id",
      named_params! {":id":id},
      |row| {
        Result::Ok(VideoEntity::new_by_file_name(
          id,
          row.get_unwrap("video_file_name"),
          row.get_unwrap("cover_file_name"),
          row.get_unwrap("dir_path"),
          row.get_unwrap("base_index"),
        ))
      },
    )
    .unwrap();

  (StatusCode::OK, Json(Option::Some(video_entity)))
}

pub async fn video_meta_info_handler(
  Path(sub_dir): Path<String>,
) -> (StatusCode, Json<Option<VideoMetaInfo>>) {
  println!("{}", sub_dir);
  let path = std::path::Path::new(&sub_dir);
  let (video_name, file_size): (String, u64) = if path.is_file() {
    let file_size = path.metadata().map_or_else(|_| 0, |m| m.len());
    (sub_dir, file_size)
  } else {
    let ret = fs::read_dir(&sub_dir);
    let file_entry_opt: Option<DirEntry> = match ret {
      Ok(dir) => dir
        .map(|res| res.unwrap())
        .find(|res| res.file_name().into_string().unwrap().ends_with(".mp4")),
      Err(_) => {
        return (StatusCode::NOT_FOUND, Json(Option::None));
      }
    };

    let file_entry = match file_entry_opt {
      Some(file_entry) => file_entry,
      None => {
        return (StatusCode::NOT_FOUND, Json(Option::None));
      }
    };

    let file_size = file_entry.metadata().map_or_else(|_| 0, |m| m.len());
    let video_name: String = match file_entry.path().into_os_string().into_string() {
      Ok(video_name) => video_name,
      Err(_) => {
        return (StatusCode::NOT_FOUND, Json(Option::None));
      }
    };

    println!("{}", video_name);
    (video_name, file_size)
  };

  let mut meta_info = parse_video_meta_info(&video_name);
  meta_info.size = file_size;

  (StatusCode::OK, Json(Some(meta_info)))
}

pub async fn generate_video_snapshot(Path(sub_dir): Path<String>) -> StatusCode {
  println!("{}", sub_dir);
  let path = std::path::Path::new(&sub_dir);
  let names: Vec<(String, String)> = if path.is_file() {
    let parent = path.parent().unwrap();
    let video_name = path.file_name().unwrap();
    let image_name =
      String::from(parent.to_str().unwrap()) + "/" + video_name.to_str().unwrap() + ".png";
    vec![(sub_dir, image_name)]
  } else {
    let ret = fs::read_dir(&sub_dir);
    if ret.is_err() {
      return StatusCode::NOT_FOUND;
    }
    let file_entrys: Vec<(String, String)> = ret
      .unwrap()
      .map(|res| res.unwrap())
      .filter(|res| res.file_name().into_string().unwrap().ends_with(".mp4"))
      .map(|f| {
        let video_name = f.path().into_os_string().into_string().unwrap();
        let mut img_name = video_name.clone();
        img_name.push_str(".png");
        (video_name, img_name)
      })
      .collect();
    file_entrys
  };

  let names_iter = names.iter();
  names_iter.for_each(|names| {
    let video_name = &names.0;
    let image_name = &names.1;
    #[cfg(reallink)]
    unsafe {
      let video_name = CString::new(video_name.as_str()).unwrap();
      let img_name = CString::new(image_name.as_str()).unwrap();
      frame_decode_with_param(video_name.as_ptr(), img_name.as_ptr());
    }
    #[cfg(mocklink)]
    {
      let video_name = CString::new(video_name.as_str()).unwrap();
      let img_name = CString::new(image_name.as_str()).unwrap();
      frame_decode_with_param(video_name.as_ptr(), img_name.as_ptr());
    }
  });

  StatusCode::OK
}

pub async fn all_duplicate_cover(
  Query(params): Query<HashMap<String, String>>,
) -> (StatusCode, Json<Vec<DuplicateCoverEntity>>) {
  let dir_path_param = params.get("dir_path");
  let base_index_param = params.get("base_index");
  let conn1 = get_sqlite_connection();

  let mut where_state = String::from("");

  let mut query_param: Vec<&String> = vec![];
  if dir_path_param.is_some() || base_index_param.is_some() {
    where_state.push_str(" where ");
  }

  if dir_path_param.is_some() {
    where_state.push_str(" vi.dir_path = ? ");
    query_param.push(dir_path_param.unwrap());
  }
  if base_index_param.is_some() {
    if query_param.len() > 0 {
      where_state.push_str(" and ");
    }
    where_state.push_str(" vi.base_index = ? ");
    query_param.push(base_index_param.unwrap());
  }

  let sql = format!("select count, cover_file_name, dir_path from(
      select count(vi.id) as count, vi.cover_file_name, vi.dir_path from video_info  vi {} group by cover_file_name, dir_path
    ) t where t.count > 1", where_state);

  let mut stmt = conn1.prepare(&sql).unwrap();

  let mut duplicate_entity_list: Vec<DuplicateCoverEntity> = stmt
    .query_map(params_from_iter(query_param.iter()), |row| {
      Ok(DuplicateCoverEntity {
        count: row.get_unwrap(0),
        cover_file_name: row.get_unwrap(1),
        video_info_list: vec![],
        dir_path: row.get_unwrap("dir_path"),
      })
    })
    .unwrap()
    .map(|it| it.unwrap())
    .collect();

  for duplicate_entity in &mut duplicate_entity_list {
    let mut stmt = conn1
      .prepare(
        "select 
        id, 
        video_file_name, 
        cover_file_name, 
        dir_path, 
        base_index, 
        designation_char, 
        designation_num 
      from 
        video_info 
      where cover_file_name=:cover_file_name and dir_path=:dir_path ",
      )
      .unwrap();

    let selected_video: Vec<VideoEntity> = stmt
      .query_map(
        named_params! {
          ":cover_file_name": &duplicate_entity.cover_file_name,
          ":dir_path": &duplicate_entity.dir_path
        },
        |row| {
          Ok(VideoEntity::new_by_for_duplicate_cover(
            row.get_unwrap(0),
            row.get_unwrap(1),
            row.get_unwrap(2),
            row.get_unwrap(3),
            row.get_unwrap(4),
            row.get_unwrap(5),
            row.get_unwrap(6),
          ))
        },
      )
      .unwrap()
      .map(|it| it.unwrap())
      .collect();
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

  let mut duplicate_entity_list: Vec<DuplicateEntity> = stmt
    .query_map(named_params! {}, |row| {
      Ok(DuplicateEntity {
        count: row.get_unwrap(0),
        designation_char: row.get_unwrap(1),
        designation_num: row.get_unwrap(2),
        video_info_list: vec![],
      })
    })
    .unwrap()
    .map(|it| it.unwrap())
    .collect();

  for duplicate_entity in &mut duplicate_entity_list {
    let mut stmt = conn1
      .prepare(
        r"select 
        id, video_file_name, cover_file_name, 
        dir_path, base_index, rate,
        video_size, cover_size, width, height,duration,frame_rate,video_frame_count
      from 
        video_info 
      where 
        designation_char=:char and designation_num=:num ",
      )
      .unwrap();

    let selected_video: Vec<VideoEntity> = stmt
      .query_map(
        named_params! {
          ":char" : &duplicate_entity.designation_char,
          ":num" : &duplicate_entity.designation_num,
        },
        |row| {
          Ok(VideoEntity::new_for_meta_info(
            row.get_unwrap(0),
            row.get_unwrap(1),
            row.get_unwrap(2),
            row.get_unwrap(3),
            row.get_unwrap(4),
            row.get_unwrap("video_size"),
            row.get_unwrap("cover_size"),
            row.get_unwrap("rate"),
            row.get_unwrap("height"),
            row.get_unwrap("width"),
            row.get_unwrap("frame_rate"),
            row.get_unwrap("video_frame_count"),
            row.get_unwrap("duration"),
          ))
        },
      )
      .unwrap()
      .map(|it| it.unwrap())
      .collect();
    duplicate_entity.video_info_list = selected_video;
  }

  (StatusCode::OK, Json(duplicate_entity_list))
}

pub async fn designation_search(
  Path(designation_ori): Path<String>,
) -> (StatusCode, HeaderMap, Json<Vec<VideoEntity>>) {
  let designation = parse_designation(&designation_ori);
  let conn1 = get_sqlite_connection();

  let mut stmt = conn1.prepare( "
    select 
      id, video_file_name, cover_file_name, rate, video_size, base_index, dir_path 
    from 
      video_info 
    where 
      designation_char=:char and designation_num=:num")
    .unwrap();

  let selected_video_iter = stmt
    .query_map(
      named_params! {
          ":char" : designation.char_final.unwrap(),
          ":num" : designation.num_final.unwrap(),
      },
      |row| {
        Ok(VideoEntity::new_for_base_info(
          row.get_unwrap(0),
          row.get_unwrap(1),
          row.get_unwrap(2),
          row.get_unwrap(4),
          row.get_unwrap(3),
          row.get_unwrap(5),
          row.get_unwrap(6),
        ))
      },
    )
    .unwrap()
    .map(|it| it.unwrap());
  let selected_video: Vec<VideoEntity> = selected_video_iter.collect();
  let mut header = HeaderMap::new();
  header.insert("Access-Control-Allow-Origin", "*".parse().unwrap());
  header.insert(
    "content-type",
    "application/json; charset=utf-8".parse().unwrap(),
  );
  (StatusCode::OK, header, Json(selected_video))
}

fn parse_dir_path(dir_path: &String) -> Result<Vec<(String, u64)>, std::io::Error> {
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

pub async fn parse_designation_handler(
  Path((base_index, sub_dir)): Path<(u32, String)>,
) -> (StatusCode, HeaderMap, Json<Vec<VideoEntity>>) {
  println!("{}", base_index);
  println!("{}", sub_dir);
  let mut sub_dir_param = String::from("/");
  sub_dir_param += &sub_dir;
  if sub_dir_param.ends_with("/") {
    sub_dir_param.truncate(sub_dir_param.len() - 1);
  }

  let sqlite_conn = get_sqlite_connection();

  let mut stmt = sqlite_conn
    .prepare(
      "select 
    id, video_file_name, cover_file_name 
  from 
    video_info 
  where 
    dir_path = :dir_path 
    and base_index=:base_index",
    )
    .unwrap();
  let selected_video: Vec<VideoEntity> = stmt
    .query_map(
      named_params! {
        ":dir_path": sub_dir_param, ":base_index": base_index
      },
      |row| {
        let designation = parse_designation(&row.get_unwrap(1));
        return Ok(VideoEntity::new_by_for_duplicate_cover(
          row.get_unwrap(0),
          row.get_unwrap(1),
          row.get_unwrap(2),
          String::new(),
          0,
          designation.char_final.unwrap(),
          designation.num_final.unwrap(),
        ));
      },
    )
    .unwrap()
    .map(|it| it.unwrap())
    .collect();

  selected_video.iter().for_each(|video| {
    let mut stmt = sqlite_conn
      .prepare(
        "update 
      video_info 
    set 
      designation_char=:char, 
      designation_num=:num 
    where 
      id=:id",
      )
      .unwrap();
    let _ = stmt.execute(named_params! {
      "char": video.designation_char.clone(),
      "num": video.designation_num.clone(),
      "id": video.id,
    });
  });

  let mut header = HeaderMap::new();
  header.insert("Access-Control-Allow-Origin", "*".parse().unwrap());
  header.insert(
    "content-type",
    "application/json; charset=utf-8".parse().unwrap(),
  );

  (StatusCode::OK, header, Json(selected_video))
}

fn query_mount_configs() -> Vec<MountConfig> {

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

pub async fn parse_meta_info_all_handler() -> StatusCode {
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
    cover_size is null or cover_size=''",
    )
    .unwrap();
  let file_names: Vec<(u32, String, String)> = stmt
    .query_map(named_params! {}, |row| {
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

  thread::spawn(move || {
    println!("thread process");

    file_names
      .into_iter()
      .for_each(|(id, video_file_name, cover_file_name)| {
        parse_and_update_meta_info_by_id(id, video_file_name, cover_file_name);
      });
  });

  StatusCode::OK
}

pub async fn parse_designation_all_handler() -> (StatusCode, HeaderMap, Json<Vec<VideoEntity>>) {
  // let mut conn = unsafe {
  //   POOL.unwrap().get_conn().unwrap()
  // };

  // let selected_video: Vec<VideoEntity> = conn.query_map(
  //   "select id, video_file_name, cover_file_name from video_info ", |(id, video_file_name, cover_file_name)| {
  //     let designation = parse_designation(&video_file_name);

  //     return VideoEntity{
  //       id,
  //       video_file_name,
  //       cover_file_name,
  //       designation_char: designation.char_final.unwrap(),
  //       designation_num: designation.num_final.unwrap(),
  //       dir_path: String::new(),
  //       base_index: 0,
  //       video_size: Option::Some(0),
  //       rate: Option::None,
  //       height:0,
  //       width: 0,
  //       frame_rate: 0,
  //       video_frame_count: 0,
  //       duration: 0,
  //     };
  //   }).unwrap();

  // selected_video.iter().for_each(|video| {
  //   let _:Vec<Row> = conn.exec("update video_info set designation_char=:char, designation_num=:num where id=:id", params! {
  //     "char" => video.designation_char.clone(),
  //     "num" => video.designation_num.clone(),
  //     "id" => video.id
  //   }).unwrap();

  // });

  let mut header = HeaderMap::new();
  header.insert("Access-Control-Allow-Origin", "*".parse().unwrap());
  header.insert(
    "content-type",
    "application/json; charset=utf-8".parse().unwrap(),
  );

  (StatusCode::OK, header, Json(vec![]))
}

pub async fn sync_mysql2sqlite_mount_config() -> (StatusCode, HeaderMap, Json<Vec<MountConfig>>) {
  // let mut conn = get_mysql_connection();
  // let sqlite_conn = get_sqlite_connection();
  // let mount_config: Vec<MountConfig> = conn.query_map(
  //   "select id, dir_path,url_prefix,api_version from mp4_base_dir ",
  //   |(id, dir_path,url_prefix,api_version)| {
  //     return MountConfig{
  //       id,
  //       dir_path,
  //       url_prefix,
  //       api_version
  //     };
  //   }).unwrap();

  // (&mount_config).into_iter().for_each(|video_entity| {
  //   sqlite_conn.execute("insert into mp4_base_dir (
  //     id, dir_path,url_prefix,api_version
  //   ) values (
  //     ?1, ?2, ?3, ?4
  //   )",
  //   rusqlite::params![video_entity.id, video_entity.dir_path, video_entity.url_prefix, video_entity.api_version]).unwrap();
  // });

  let mut header = HeaderMap::new();
  header.insert("Access-Control-Allow-Origin", "*".parse().unwrap());
  header.insert(
    "content-type",
    "application/json; charset=utf-8".parse().unwrap(),
  );

  (StatusCode::OK, header, Json(vec![]))
}

pub async fn sync_mysql2sqlite_video_info() -> (StatusCode, HeaderMap, Json<Vec<VideoEntity>>) {
  // let mut conn = get_mysql_connection();
  // let sqlite_conn = get_sqlite_connection();

  // let selected_video: Vec<VideoEntity> = conn.query_map(
  //   "select id, dir_path,base_index,rate, video_file_name, cover_file_name, designation_num, designation_char from video_info ",
  //   |(id, dir_path, base_index, rate, video_file_name, cover_file_name, designation_num, designation_char)| {
  //     return VideoEntity{
  //       id,
  //       video_file_name,
  //       cover_file_name,
  //       designation_char,
  //       designation_num,
  //       dir_path,
  //       base_index,
  //       rate,
  //       video_size: Option::Some(0),
  //       height:0,
  //       width: 0,
  //       frame_rate: 0,
  //       video_frame_count: 0,
  //       duration: 0,
  //     };
  //   }).unwrap();

  // (&selected_video).into_iter().for_each(|video_entity| {
  //   sqlite_conn.execute("insert into video_info (
  //     id, dir_path, base_index,rate, video_file_name, cover_file_name, designation_num, designation_char
  //   ) values (
  //     ?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8
  //   )", rusqlite::params![
  //     video_entity.id,
  //     video_entity.dir_path,
  //     video_entity.base_index,
  //     video_entity.rate,
  //     video_entity.video_file_name,
  //     video_entity.cover_file_name,
  //     video_entity.designation_num,
  //     video_entity.designation_char]).unwrap();
  // });

  let mut header = HeaderMap::new();
  header.insert("Access-Control-Allow-Origin", "*".parse().unwrap());
  header.insert(
    "content-type",
    "application/json; charset=utf-8".parse().unwrap(),
  );

  (StatusCode::OK, header, Json(vec![]))
}

pub fn snapshot(file_url: CString, snap_time: u64) -> SnapshotSt {
  unsafe {
    let snapshot_st = snapshot_video(file_url.as_ptr(), snap_time);

    snapshot_st
  }
}

pub async fn snapshot_handler(
  Path(sub_dir): Path<String>,
  Query(params): Query<HashMap<String, u64>>,
) -> Response {
  let video_name = CString::new(sub_dir.as_str()).unwrap();
  let time_param = params.get("time");
  let snapshot_st = snapshot(video_name, *time_param.unwrap());
  println!("snapshot_st len:{}", snapshot_st.buff_len);

  let content_type_value = String::from("image/png");
  let mut header = HeaderMap::new();
  header.insert(ACCESS_CONTROL_ALLOW_ORIGIN, "*".parse().unwrap());
  header.insert(CONTENT_TYPE, content_type_value.parse().unwrap());
  header.insert(CONTENT_LENGTH, snapshot_st.buff_len.into());

  let mut response_builder = Response::builder().status(StatusCode::OK);
  *response_builder.headers_mut().unwrap() = header;

  let bytes: Bytes = unsafe {
    let len: usize = snapshot_st.buff_len.try_into().unwrap();
    let slice = slice::from_raw_parts(snapshot_st.buff, len);
    let buff: Vec<u8> = Vec::from(slice);
    libc::free(snapshot_st.buff as *mut c_void);
    Bytes::from(buff)
  };

  let buff_stream = BuffStream { bytes, done: false };
  response_builder
    .body(Body::from_stream(buff_stream))
    .unwrap()
}

struct BuffStream {
  // snapshot_st: SnapshotSt,
  bytes: Bytes,
  done: bool,
}

impl futures_core::Stream for BuffStream {
  type Item = Result<Bytes, Error>;

  fn poll_next(
    mut self: std::pin::Pin<&mut Self>,
    _: &mut std::task::Context<'_>,
  ) -> std::task::Poll<Option<Self::Item>> {
    if self.done {
      return std::task::Poll::Ready(None);
    } else {
      self.done = true;
      return std::task::Poll::Ready(Some(Ok(self.bytes.clone())));
    }
  }
}

pub async fn init_video_handler(
  Path((base_index, sub_dir)): Path<(u32, String)>,
) -> (StatusCode, HeaderMap, Json<Vec<VideoCover>>) {
  println!("{}", base_index);
  println!("{}", sub_dir);
  let mut sub_dir_param = String::from("/");
  sub_dir_param += &sub_dir;

  let sqlite_conn = get_sqlite_connection();

  let mut stmt = sqlite_conn
    .prepare("select dir_path from mp4_base_dir where id = :id")
    .unwrap();
  let mut dir_path: String = stmt
    .query_row(named_params! {":id": base_index}, |row| {
      Ok(row.get_unwrap(0))
    })
    .unwrap();

  dir_path += &sub_dir_param;

  let file_names: Vec<(String, u64)> = parse_dir_path(&dir_path).unwrap();
  dir_path += "/";

  let video_cover_list = parse_video_cover(&file_names);

  for video_cover_entry in video_cover_list.iter() {
    let mut dir_path_tmp = dir_path.clone();
    dir_path_tmp += "/";
    dir_path_tmp += video_cover_entry.video_file_name.as_str();
    let meta_info = parse_video_meta_info(&dir_path_tmp);
    let path = std::path::Path::new(&dir_path_tmp);
    let video_size = path.metadata().map_or_else(|_| 0, |m| m.len());

    let mut dir_path_tmp = dir_path.clone();
    dir_path_tmp += "/";
    dir_path_tmp += video_cover_entry.cover_file_name.as_str();
    let path = std::path::Path::new(&dir_path_tmp);
    let cover_size = path.metadata().map_or_else(|_| 0, |m| m.len());

    let designation = parse_designation(&video_cover_entry.video_file_name);
    let exist = check_exist_by_video_file_name(
      &sub_dir_param,
      base_index,
      &video_cover_entry.video_file_name,
    );

    if !exist {
      let _ = sqlite_conn.execute("insert into video_info(
        dir_path, base_index, video_file_name, cover_file_name, designation_char, 
        designation_num, 
        video_size, width, height,duration,frame_rate,video_frame_count,cover_size
      ) values (
        :dir_path, :base_index, :video_file_name, :cover_file_name, :designation_char, :designation_num, 
        :video_size, :width, :height,:duration,:frame_rate,:video_frame_count,:cover_size
      )", named_params! {
        ":dir_path": sub_dir_param, 
        ":base_index": base_index, 
        ":video_file_name": video_cover_entry.video_file_name, 
        ":cover_file_name": video_cover_entry.cover_file_name,
        ":designation_char": designation.char_final, 
        ":designation_num": designation.num_final,
        ":video_size": video_size,
        ":cover_size": cover_size,
        ":width": meta_info.width,
        ":height": meta_info.height,
        ":duration": meta_info.duratoin,
        ":frame_rate": meta_info.frame_rate,
        ":video_frame_count": meta_info.video_frame_count,
      });
    } else {
      let _ = sqlite_conn.execute(
        "update video_info set 
        cover_file_name=:cover_file_name, 
        designation_char=:designation_char, 
        designation_num=:designation_num, 
        video_size=:video_size,
        cover_size=:cover_size,
        height=:height,
        width=:width,
        duration=:duration,
        frame_rate=:frame_rate,
        video_frame_count=:video_frame_count
      where
        dir_path=:dir_path and base_index=:base_index and video_file_name=:video_file_name
      ",
        named_params! {
          ":dir_path": sub_dir_param,
          ":base_index": base_index,
          ":video_file_name": video_cover_entry.video_file_name,
          ":cover_file_name": video_cover_entry.cover_file_name,
          ":designation_char": designation.char_final,
          ":designation_num": designation.num_final,
          ":video_size": video_size,
          ":cover_size": cover_size,
          ":width": meta_info.width,
          ":height": meta_info.height,
          ":duration": meta_info.duratoin,
          ":frame_rate": meta_info.frame_rate,
          ":video_frame_count": meta_info.video_frame_count,
        },
      );
    }
  }

  println!("{:?}", video_cover_list);

  let mut header = HeaderMap::new();
  header.insert("Access-Control-Allow-Origin", "*".parse().unwrap());
  header.insert(
    "content-type",
    "application/json; charset=utf-8".parse().unwrap(),
  );

  (StatusCode::OK, header, Json(video_cover_list))
}

fn check_exist_by_video_file_name(
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

pub async fn move_cover() {

  let mount_configs = query_mount_configs();

  let sqlite_conn = get_sqlite_connection();

  let mut stmt = sqlite_conn.prepare("
  select
    id, video_file_name, cover_file_name, dir_path, base_index
  from
    video_info
  where
    moved isnull or moved == 0
  ").unwrap();
  let unmoved: Vec<VideoEntity> = stmt.query_map({}, |row| {
    Ok(VideoEntity::new_by_file_name(
          row.get_unwrap("id"),
          row.get_unwrap("video_file_name"),
          row.get_unwrap("cover_file_name"),
          row.get_unwrap("dir_path"),
          row.get_unwrap("base_index"),
    ))
  }).unwrap().map(|it|it.unwrap()).collect();
  let jh = thread::spawn(move || {
    unmoved.into_iter().for_each(|video_entity| {
      let (_, cover_path, dir_path) = video_entity_to_file_path(&video_entity, &mount_configs);
      // TODO: copy cover image
      let mut target_dir = mount_configs[0].dir_path.clone();
      target_dir.push_str("/covers");
      target_dir.push_str(&dir_path);
      let target_dir_path = std::path::Path::new(&target_dir);
      if !target_dir_path.exists() {
        DirBuilder::new()
          .recursive(true)
          .create(target_dir_path).unwrap();
      }

      let mut target_cover_file = target_dir;
      
      target_cover_file.push_str("/");
      target_cover_file.push_str(&video_entity.cover_file_name);

      if std::path::Path::new(&cover_path).exists() {
        println!("copy cover path: {:?} to {:?}", cover_path, target_cover_file);
        let _ = fs::copy(cover_path, target_cover_file);
        let _ = get_sqlite_connection().execute(
          "update video_info set moved = 1 where id = :id",
          named_params! {":id": video_entity.id}
        );
      }

    });
  });
  let _ = jh.join();

}

fn video_entity_to_file_path(video_entity: &VideoEntity, mount_configs: &Vec<MountConfig>) -> (String, String, String) {
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