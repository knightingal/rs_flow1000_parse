use rs_flow1000_parse::base_lib::{linux_init, parse_and_update_meta_info_by_id, video_file_path_by_id};

fn main() {

  linux_init();
  let id = 1;
  let file_names = video_file_path_by_id(id);
  println!("file_names:{:?}", file_names);

  file_names
    .into_iter()
    .for_each(|(id, video_file_name, cover_file_name)| {
      parse_and_update_meta_info_by_id(id, video_file_name, cover_file_name);
    });
}