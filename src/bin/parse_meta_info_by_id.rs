use std::env;

use rs_flow1000_parse::base_lib::{os_init, parse_and_update_meta_info_by_id, video_file_path_by_id};

fn main() {

  tracing_subscriber::fmt::init();

  let args: Vec<String> = env::args().collect();
  if args.len() < 2 {
    tracing::error!("invalid args. input args to indicate video id such as \"3\"");
    return;
  }
  os_init();
  let id = args[1].parse().unwrap();
  let file_names = video_file_path_by_id(id);
  tracing::info!("file_names:{:?}", file_names);

  file_names
    .into_iter()
    .for_each(|(id, video_file_name, cover_file_name, _)| {
      parse_and_update_meta_info_by_id(id, video_file_name, cover_file_name);
    });
}