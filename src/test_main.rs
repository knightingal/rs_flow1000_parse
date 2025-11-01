#[cfg(test)]
mod tests {
  use std::{ffi::{c_void, CString}, fs::File, io::{Read, Seek, SeekFrom, Write}};

use rusqlite::{Connection, named_params};
use sysinfo::System;

use crate::{entity::VideoEntity, get_sqlite_connection, handles::{IS_LINUX, query_mount_configs, snapshot, video_entity_to_file_path}, linux_init};

// use crate::handles::move_cover;

  #[test]
  fn it_works() {
    assert_eq!(2 + 2, 4);
  }

  #[test]
  fn move_cover_test() {
    // move_cover();

    let video_name = CString::new("/home/knightingal/demo_video.mp4").unwrap();
    let snapshot_st = snapshot(video_name, 10);

    unsafe { libc::free(snapshot_st.buff as *mut c_void) };
    println!("snapshot_st len:{}", snapshot_st.buff_len)
  }

  #[test]
  fn create_large_file() -> std::io::Result<()> {
    let mut f = File::create("testfile.hex")?;
    let mut i: u32 = 0;
    loop {
      if i <= 0xffff {
        let u0: u8= (i >> 24) as u8;
        let u1: u8= ((i >> 16) & 0xff) as u8;
        let u2: u8= ((i >> 8) & 0xff) as u8;
        let u3: u8= (i & 0xff) as u8;
        let buf = [u0, u1, u2, u3];
        let _ = f.write(&buf);
        if i & 0xffff == 0 {
          println!("{:08x}", i);
        }

      } else {
        break;
      }
      i = i + 1;
    }

    Ok(())
  }

  #[test]
  fn seek_file() -> std::io::Result<()> {
    let mut f = File::open("testfile.bin")?;
    let pos : SeekFrom = SeekFrom::Start(0x3ffffffc);
    let _ = f.seek(pos);
    let mut buf:[u8; 4] = [0,0,0,0];
    let _ = f.read(& mut buf);
    println!("{:02x},{:02x},{:02x},{:02x},", buf[0], buf[1], buf[2], buf[3]);
    Ok(())
  }



  #[test]
  fn test_concat_cover() {

    linux_init();

    let mount_config_list = query_mount_configs();
    let base_mount = mount_config_list.iter().find(|it| it.id == 1).unwrap();
    let dir_name = "/202402";
    let sqlite_conn: Connection = get_sqlite_connection();
    let mut stmt = sqlite_conn.prepare(
      "SELECT 
        id, video_file_name, base_index, dir_path, cover_file_name, cover_size, cover_offset
      FROM 
        video_info 
      WHERE 
        dir_path = :dir_path").unwrap();
    let ids: Vec<(
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
    println!("ids:{:?}", ids);
    

  }
}