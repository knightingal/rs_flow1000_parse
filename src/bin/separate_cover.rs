use std::{fs::File, io::{Read, Write}, path::Path};

use rs_flow1000_parse::{base_lib::{get_sqlite_connection, linux_init, query_mount_configs, video_entity_to_file_path}, entity::VideoEntity};
use rusqlite::{Connection, named_params};


fn main() {
  println!("separate cover!");

  linux_init();

  let mount_config_list = query_mount_configs();
  let base_mount = mount_config_list.iter().find(|it| it.id == 1).unwrap();
  let dir_name = "/202512";
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



  let mut concat_file = File::open(concat_path_name).unwrap();
  let mut header: [u8; 4]  = [0, 0, 0, 0];
  let _ = concat_file.read_exact(& mut header);
  if !header.eq(&[0xca, 0xfe, 0xba, 0xbe]) {
    println!("Error file");
    return;
  }

  let mut read_offset: u64 = 4;


  covers.iter().for_each(|(_, _video_file_name, cover_file_name, cover_size, _cover_offset)| {

    let f_err = File::create(cover_file_name);
    if f_err.is_err() {
      println!("open cover file error: {}", cover_file_name);
      return;
    }
    let mut f = f_err.unwrap();
      
    let mut buf: Vec<u8> = vec![0; *cover_size as usize];
    let _ = concat_file.read_exact(& mut buf);
    let _ = f.write(&mut buf);
    f.flush().unwrap();
    read_offset += *cover_size;
  });

  concat_file.flush().unwrap()

}