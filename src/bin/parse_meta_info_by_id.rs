use std::env;

use rs_flow1000_parse::base_lib::{os_init, refresh_video_and_cover_by_id};

fn main() {

  tracing_subscriber::fmt::init();

  let args: Vec<String> = env::args().collect();
  if args.len() < 2 {
    tracing::error!("invalid args. input args to indicate video id such as \"3\"");
    return;
  }
  os_init();
  let id = args[1].parse().unwrap();
  refresh_video_and_cover_by_id(id);
}