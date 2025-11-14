use rs_flow1000_parse::{base_lib::{check_exist_by_video_file_name, get_sqlite_connection, linux_init, parse_dir_path}, designation::parse_designation, video_name_util::{parse_video_cover, parse_video_meta_info}};
use rusqlite::named_params;

fn main() {
  println!("import monthly videos!");

  linux_init();

  let base_index: u32 = 1;
  let sub_dir: String = String::from("/201710");

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
}