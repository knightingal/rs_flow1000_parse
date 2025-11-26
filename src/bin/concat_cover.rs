use std::{fs::File, io::{Read, Seek, SeekFrom, Write}, path::Path};

use rs_flow1000_parse::{base_lib::{get_sqlite_connection, linux_init, query_mount_configs, video_entity_to_file_path}, entity::VideoEntity};
use rusqlite::{Connection, named_params};


fn main() {
  println!("concat cover!");

  linux_init();

  let mount_config_list = query_mount_configs();
  let base_mount = mount_config_list.iter().find(|it| it.id == 1).unwrap();
  let dir_name = "/202411";
  let sqlite_conn: Connection = get_sqlite_connection();
  let mut stmt = sqlite_conn.prepare(
    "select 
      id, video_file_name, base_index, dir_path, cover_file_name, cover_size, cover_offset
    FROM 
      video_info 
    WHERE 
      dir_path = :dir_path").unwrap();

  let covers: Vec<(
      u32, 
      String, 
      String, 
      u64, 
      u64
  )> = stmt.query_map(named_params! {":dir_path": dir_name}, |row| {
    let video_file_name: String = row.get_unwrap("video_file_name");
    let cover_file_name: String = row.get_unwrap("cover_file_name");
    let dir_path: String = row.get_unwrap("dir_path");
    let base_index: u32 = row.get_unwrap("base_index");
    let id: u32 = row.get_unwrap("id");
    let cover_size: u64 = row.get_unwrap("cover_size");
    let cover_offset: u64 = row.get_unwrap("cover_offset");
    let (video_full_name, cover_full_name, _) = video_entity_to_file_path(&VideoEntity::new_by_file_name(
      id, video_file_name, cover_file_name, dir_path, base_index
    ), &mount_config_list);

    Result::Ok((
      id, 
      video_full_name, 
      cover_full_name, 
      cover_size, 
      cover_offset
    ))
  }).unwrap().map(|result| {
    result.unwrap()
  }).collect();
  println!("ids:{:?}", covers);

  let concat_file_name = base_mount.dir_path.clone() + "/covers" + covers[0].2.as_str();
  let concat_path = Path::new(&concat_file_name).parent().unwrap();
  let concat_path_name = concat_path.join("main.class");

  if concat_path_name.exists() {
    std::fs::remove_file(&concat_path_name).unwrap();
  }


  let mut concat_file = File::create_new(concat_path_name).unwrap();
  let header: [u8; 4] = [0xca, 0xfe, 0xba, 0xbe]; // "CAFEBABE"
  let _ = concat_file.write(&header);

  let mut write_offset: u64 = 4;

  let mut stmt = sqlite_conn.prepare(
    "update 
      video_info 
    set 
      cover_offset = :cover_offset 
    where 
      id = :id").unwrap();

  covers.iter().for_each(|(id, _video_file_name, cover_file_name, cover_size, _cover_offset)| {
    stmt.execute(named_params! {
      ":cover_offset": write_offset,
      ":id": *id
    }).unwrap();

    let f_err = File::open(cover_file_name);
    if f_err.is_err() {
      println!("open cover file error: {}", cover_file_name);
      return;
    }
    let mut f = f_err.unwrap();
      
    let mut buf: Vec<u8> = vec![0; *cover_size as usize];
    let _ = f.seek(SeekFrom::Start(0));
    let _ = f.read_exact(&mut buf);
    let _ = concat_file.write_all(&buf);
    write_offset += *cover_size;
  });

  concat_file.flush().unwrap()

}