use std::{
  cmp::Ordering,
  env,
  fs::{self, DirEntry},
  future::Future,
  pin::Pin,
  sync::{Arc, Mutex},
  task::{Context, Poll},
};

use axum::{extract::Path, Json};
use hyper::{
  header::{ACCESS_CONTROL_ALLOW_ORIGIN, CONTENT_TYPE},
  HeaderMap, StatusCode,
};
use rusqlite::{named_params, params_from_iter, Connection};
use tokio::task;

use crate::{entity::*, handles::IS_LINUX};

fn get_sqlite_connection() -> Connection {
  let db_path_env = env::var("DB_PATH")
    .unwrap_or_else(|_| String::from("/home/knightingal/source/keys/mp41000.db"));
  let conn = Connection::open(db_path_env).unwrap();
  return conn;
}

pub async fn video_info_handler(
  Path((base_index, sub_dir)): Path<(u32, String)>,
) -> (StatusCode, HeaderMap, Json<Vec<VideoEntity>>) {
  let mut sub_dir_param = String::from("/");
  sub_dir_param += &sub_dir;
  if sub_dir_param.ends_with("/") {
    sub_dir_param.truncate(sub_dir_param.len() - 1);
  }

  let sqlite_conn = get_sqlite_connection();

  let mut stmt = sqlite_conn.prepare("select id, video_file_name, cover_file_name, rate, video_size from video_info where dir_path = :dir_path and base_index=:base_index").unwrap();
  let selected_video_iter = stmt
    .query_map(
      named_params! {":dir_path": sub_dir_param.as_str(),":base_index": base_index},
      |row| {
        Ok(VideoEntity::new_for_base_info(
          row.get_unwrap(0),
          row.get_unwrap(1),
          row.get_unwrap(2),
          row.get_unwrap(4),
          row.get_unwrap(3),
        ))
      },
    )
    .unwrap()
    .map(|it| it.unwrap());

  let selected_video: Vec<VideoEntity> = selected_video_iter.collect();

  let mut header = HeaderMap::new();
  header.insert(ACCESS_CONTROL_ALLOW_ORIGIN, "*".parse().unwrap());
  header.insert(
    CONTENT_TYPE,
    "application/json; charset=utf-8".parse().unwrap(),
  );

  (StatusCode::OK, header, Json(selected_video))
}

pub async fn mount_config_handler() -> (StatusCode, HeaderMap, Json<Vec<MountConfig>>) {
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

  let mut header = HeaderMap::new();
  header.insert(ACCESS_CONTROL_ALLOW_ORIGIN, "*".parse().unwrap());
  header.insert(
    CONTENT_TYPE,
    "application/json; charset=utf-8".parse().unwrap(),
  );

  (StatusCode::OK, header, Json(mount_config_list))
}

pub async fn mp4_dir_handler1(
  Path(base_index): Path<u32>,
) -> (StatusCode, HeaderMap, Json<Vec<String>>) {
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
  let dir_path: String = stmt
    .query_row(named_params! {":id": base_index}, |row| {
      Ok(row.get_unwrap(0))
    })
    .unwrap();

  let file_names: Vec<(String, u64)> = parse_dir_path(&dir_path).unwrap();

  let mut header = HeaderMap::new();
  header.insert(ACCESS_CONTROL_ALLOW_ORIGIN, "*".parse().unwrap());
  header.insert(
    CONTENT_TYPE,
    "application/json; charset=utf-8".parse().unwrap(),
  );

  (
    StatusCode::OK,
    header,
    Json(file_names.into_iter().map(|f| f.0).collect()),
  )
}

pub async fn mp4_dir_handler(
  Path((base_index, sub_dir)): Path<(u32, String)>,
) -> (StatusCode, HeaderMap, Json<Vec<String>>) {
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
  let mut dir_path: String = stmt
    .query_row(named_params! {":id": base_index}, |row| {
      Ok(row.get_unwrap(0))
    })
    .unwrap();

  dir_path += "/";
  dir_path += &sub_dir;

  let file_names = parse_dir_path(&dir_path);
  let mut header = HeaderMap::new();
  header.insert(ACCESS_CONTROL_ALLOW_ORIGIN, "*".parse().unwrap());
  header.insert(
    CONTENT_TYPE,
    "application/json; charset=utf-8".parse().unwrap(),
  );

  (
    StatusCode::OK,
    header,
    Json(file_names.unwrap().into_iter().map(|f| f.0).collect()),
  )
}

pub async fn video_rate(
  Path((id, rate)): Path<(u32, u32)>,
) -> (StatusCode, HeaderMap, Json<VideoEntity>) {
  let sqlite_conn = get_sqlite_connection();

  sqlite_conn
    .execute(
      "update video_info set rate=?1 where id=?2",
      rusqlite::params![rate, id],
    )
    .unwrap();
  let result: Result<VideoEntity, _> = sqlite_conn.query_row(
    "select id, video_file_name, cover_file_name, rate from video_info where id = :id ",
    named_params! {
        ":id" : id,
    },
    |row| {
      Result::Ok(VideoEntity::new_for_base_info(
        row.get_unwrap(0),
        row.get_unwrap(1),
        row.get_unwrap(2),
        Option::Some(0),
        row.get_unwrap(3),
      ))
    },
  );

  let mut header = HeaderMap::new();
  header.insert(ACCESS_CONTROL_ALLOW_ORIGIN, "*".parse().unwrap());
  header.insert(
    CONTENT_TYPE,
    "application/json; charset=utf-8".parse().unwrap(),
  );

  (StatusCode::OK, header, Json(result.unwrap().clone()))
}

pub async fn add_tag(Path(tag_name): Path<String>) -> (StatusCode, HeaderMap, Json<TagEntity>) {
  let sqlite_conn = get_sqlite_connection();

  let mut stmt = sqlite_conn
    .prepare("select tag from tag where tag = :tag")
    .unwrap();
  let exist = stmt
    .exists(named_params! {":tag": tag_name})
    .unwrap_or(false);

  if !exist {
    let _ = sqlite_conn.execute(
      "insert into tag (tag) values (:tag)",
      named_params! {":tag": tag_name},
    );
  }

  let tag_entity: TagEntity = sqlite_conn
    .query_row(
      "select id, tag from tag where tag=:tag",
      named_params! {":tag": tag_name},
      |row| {
        Result::Ok(TagEntity {
          id: row.get_unwrap("id"),
          tag: row.get_unwrap("tag"),
        })
      },
    )
    .unwrap();

  let mut header = HeaderMap::new();
  header.insert(ACCESS_CONTROL_ALLOW_ORIGIN, "*".parse().unwrap());
  header.insert(
    CONTENT_TYPE,
    "application/json; charset=utf-8".parse().unwrap(),
  );

  (StatusCode::OK, header, Json(tag_entity))
}

pub async fn bind_tag(Path((tag_id, video_id)): Path<(u32, u32)>) -> (StatusCode, HeaderMap) {
  println!("tag_id:{}, video_id:{}", tag_id, video_id);

  let sqlite_conn = get_sqlite_connection();

  let ret = sqlite_conn.execute(
    "insert into video_tag 
  (video_id, tag_id) values (:video_id, :tag_id)",
    named_params! {":tag_id": tag_id, ":video_id": video_id},
  );
  if ret.is_err() {
    let mut header = HeaderMap::new();
    header.insert(ACCESS_CONTROL_ALLOW_ORIGIN, "*".parse().unwrap());
    header.insert(
      CONTENT_TYPE,
      "application/json; charset=utf-8".parse().unwrap(),
    );

    (StatusCode::INTERNAL_SERVER_ERROR, header)
  } else {
    let mut header = HeaderMap::new();
    header.insert(ACCESS_CONTROL_ALLOW_ORIGIN, "*".parse().unwrap());
    header.insert(
      CONTENT_TYPE,
      "application/json; charset=utf-8".parse().unwrap(),
    );

    (StatusCode::OK, header)
  }
}


pub async fn unbind_tag(Path((tag_id, video_id)): Path<(u32, u32)>) -> (StatusCode, HeaderMap) {
  println!("tag_id:{}, video_id:{}", tag_id, video_id);

  let sqlite_conn = get_sqlite_connection();

  let ret = sqlite_conn.execute(
    "delete from video_tag where video_id=:video_id and tag_id=:tag_id",
    named_params! {":tag_id": tag_id, ":video_id": video_id},
  );
  if ret.is_err() {
    let mut header = HeaderMap::new();
    header.insert(ACCESS_CONTROL_ALLOW_ORIGIN, "*".parse().unwrap());
    header.insert(
      CONTENT_TYPE,
      "application/json; charset=utf-8".parse().unwrap(),
    );

    (StatusCode::INTERNAL_SERVER_ERROR, header)
  } else {
    let mut header = HeaderMap::new();
    header.insert(ACCESS_CONTROL_ALLOW_ORIGIN, "*".parse().unwrap());
    header.insert(
      CONTENT_TYPE,
      "application/json; charset=utf-8".parse().unwrap(),
    );

    (StatusCode::OK, header)
  }
}

pub async fn query_videos_by_tag(
  Path(tag_id): Path<u32>,
) -> (StatusCode, HeaderMap, Json<Vec<VideoEntity>>) {
  let sqlite_conn = get_sqlite_connection();

  let mut stmt = sqlite_conn
    .prepare("select id, video_id from video_tag where tag_id = :tag_id")
    .unwrap();
  let video_id_vec: Vec<u32> = stmt
    .query_map(named_params! {":tag_id": tag_id}, |row| {
      Ok(row.get_unwrap("video_id"))
    })
    .unwrap()
    .map(|it| it.unwrap())
    .collect();

  let vars = repeat_vars(video_id_vec.len());
  let sql = format!("select id, video_file_name, cover_file_name, rate from video_info where id in ({})", vars);

  let mut stmt = sqlite_conn
    .prepare(&sql)
    .unwrap();
  let video_entity_vec: Vec<VideoEntity> = stmt.query_map(params_from_iter(video_id_vec), |row| {
      Result::Ok(VideoEntity::new_for_base_info(
        row.get_unwrap(0),
        row.get_unwrap(1),
        row.get_unwrap(2),
        Option::Some(0),
        row.get_unwrap(3),
      ))
  }).unwrap().map(|it|it.unwrap()).collect();

  let mut header = HeaderMap::new();
  header.insert(ACCESS_CONTROL_ALLOW_ORIGIN, "*".parse().unwrap());
  header.insert(
    CONTENT_TYPE,
    "application/json; charset=utf-8".parse().unwrap(),
  );

  (StatusCode::OK, header, Json::from(video_entity_vec))
}

fn repeat_vars(count: usize) -> String {
  assert_ne!(count, 0);
  let mut s = "?,".repeat(count);
  // Remove trailing comma
  s.pop();
  s
}

pub async fn query_tags_by_video(
  Path(video_id): Path<u32>,
) -> (StatusCode, HeaderMap, Json<Vec<u32>>) {
  let sqlite_conn = get_sqlite_connection();

  let mut stmt = sqlite_conn
    .prepare("select id, tag_id from video_tag where video_id = :video_id")
    .unwrap();
  let tag_vec: Vec<u32> = stmt
    .query_map(named_params! {":video_id": video_id}, |row| {
      Ok(row.get_unwrap("tag_id"))
    })
    .unwrap()
    .map(|it| it.unwrap())
    .collect();

  let mut header = HeaderMap::new();
  header.insert(ACCESS_CONTROL_ALLOW_ORIGIN, "*".parse().unwrap());
  header.insert(
    CONTENT_TYPE,
    "application/json; charset=utf-8".parse().unwrap(),
  );

  (StatusCode::OK, header, Json::from(tag_vec))
}

pub async fn statistic_handle() -> (StatusCode, HeaderMap, Json<StatisticEntity>) {
  let sqlite_conn = get_sqlite_connection();
  let mut stmt = sqlite_conn
    .prepare("select video_size, cover_size from video_info")
    .unwrap();
  let sizes: Vec<(u64, u64)> = stmt
    .query_map({}, |row| {
      Ok((row.get_unwrap("video_size"), row.get_unwrap("cover_size")))
    })
    .unwrap()
    .map(|it| it.unwrap())
    .collect();

  let sum = sizes
    .into_iter()
    .reduce(|acc, e| (acc.0 + e.0, acc.1 + e.1))
    .unwrap();
  let statistic = StatisticEntity {
    video_size: sum.0,
    cover_size: sum.1,
  };

  let mut header = HeaderMap::new();
  header.insert(ACCESS_CONTROL_ALLOW_ORIGIN, "*".parse().unwrap());
  header.insert(
    CONTENT_TYPE,
    "application/json; charset=utf-8".parse().unwrap(),
  );
  (StatusCode::OK, header, Json::from(statistic))
}

pub fn query_tags() -> QueryTagsFuture {
  QueryTagsFuture {
    st: Arc::new(Mutex::new(St {
      done: false,
      reps: vec![],
    })),
  }
}

pub struct QueryTagsFuture {
  st: Arc<Mutex<St>>,
}

struct St {
  done: bool,
  reps: Vec<TagEntity>,
}

impl Future for QueryTagsFuture {
  type Output = (StatusCode, HeaderMap, Json<Vec<TagEntity>>);

  fn poll(self: Pin<&mut Self>, ctx: &mut Context<'_>) -> Poll<Self::Output> {
    let st = self.st.lock().unwrap();
    if st.done == true {
      let mut header = HeaderMap::new();
      header.insert(ACCESS_CONTROL_ALLOW_ORIGIN, "*".parse().unwrap());
      header.insert(
        CONTENT_TYPE,
        "application/json; charset=utf-8".parse().unwrap(),
      );
      Poll::Ready((StatusCode::OK, header, Json(st.reps.clone())))
    } else {
      let st = self.st.clone();
      let waker = ctx.waker().clone();

      task::spawn(async move {
        let sqlite_conn = get_sqlite_connection();

        let mut stmt = sqlite_conn.prepare("select id, tag from tag").unwrap();

        let tags: Vec<TagEntity> = stmt
          .query_map({}, |row| {
            Result::Ok(TagEntity {
              id: row.get_unwrap("id"),
              tag: row.get_unwrap("tag"),
            })
          })
          .unwrap()
          .map(|it| it.unwrap())
          .collect();
        let mut st = st.lock().unwrap();
        st.done = true;
        st.reps = tags;
        waker.wake();
      });
      Poll::Pending
    }
  }
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
