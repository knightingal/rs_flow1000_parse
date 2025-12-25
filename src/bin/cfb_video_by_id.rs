use std::ffi::c_char;
use std::{ffi::CString, fs::DirBuilder};

use rs_flow1000_parse::base_lib::init_key;
use rs_flow1000_parse::{
  base_lib::{get_sqlite_connection, linux_init, query_mount_configs, IS_LINUX},
  entity::VideoEntity,
};
use rusqlite::named_params;

#[cfg(reallink)]
#[link(name = "cfb_decode")]
extern "C" {
  fn cfb_file_streaming_v2(
    w: *const u32,
    iv: *const u8,
    input_filename: *const c_char,
    output_filename: *const c_char,
  ) -> i32;
}

fn main() {
  let id = 985;

  linux_init();

  init_key();

  tracing::debug!("cfb_video_by_id handler: id: {}", id);

  let mount_configs = query_mount_configs();

  let sqlite_conn = get_sqlite_connection();
  let mut sql = String::from("select vi.id, ");
  let dir_path_name: &str;
  unsafe {
    dir_path_name = if *IS_LINUX.unwrap() {
      "dir_path"
    } else {
      "win_dir_path"
    }
  }
  sql += "mbd.";
  sql += dir_path_name;
  sql += " as mount_path, vi.video_file_name, vi.base_index, mbd.url_prefix, mbd.api_version, vi.dir_path 
    from video_info vi left join mp4_base_dir mbd on vi.base_index = mbd.id where vi.id = :id";
  let video_entity = sqlite_conn
    .query_row(sql.as_str(), named_params! {":id": id}, |row| {
      let mut dir_path: String = row.get_unwrap("mount_path");
      dir_path.push_str(row.get_unwrap::<_, String>("dir_path").as_str());
      Ok(VideoEntity::new_by_file_name(
        row.get_unwrap("id"),
        row.get_unwrap("video_file_name"),
        String::new(),
        dir_path,
        row.get_unwrap("base_index"),
      ))
    })
    .unwrap();
  let file_path = video_entity.dir_path + "/" + video_entity.video_file_name.as_str();

  let mut target_dir = mount_configs[0].dir_path.clone();
  target_dir.push_str("/cfb/");
  let mut buffer = itoa::Buffer::new();
  let base_index_str = buffer.format(video_entity.base_index);
  target_dir.push_str(base_index_str);
  let last_index = file_path.rfind('/').unwrap();
  let (parenet_dir, file_name) = file_path.split_at(last_index);
  target_dir.push_str(parenet_dir);

  let target_dir_path = std::path::Path::new(&target_dir);
  if !target_dir_path.exists() {
    DirBuilder::new()
      .recursive(true)
      .create(target_dir_path)
      .unwrap();
  }

  let mut target_file_path = target_dir;
  target_file_path.push_str(file_name);
  target_file_path.push_str(".bin");

  // let file_name = CString::new(file_name).unwrap();
  let input_file_path: CString = CString::new(file_path.as_str()).unwrap();

  let iv = "2021000120210001";

  let move_target_file_path: CString = CString::new(target_file_path.as_str()).unwrap();

  unsafe {
    cfb_file_streaming_v2(
      0 as *const u32,
      iv.as_ptr(),
      input_file_path.as_ptr(),
      move_target_file_path.as_ptr(),
    );
  }

  let update_video_cfb_sql = "update video_info set cfb = 1 where id = :id";
  sqlite_conn
    .execute(update_video_cfb_sql, named_params! {":id": id})
    .unwrap();

  tracing::debug!("target_file_path {}", target_file_path);
  tracing::debug!("file_path {}", file_path);
}
