use rs_flow1000_parse::{base_lib::{concat_cover, os_init,} };
use std::env;


fn main() {
  tracing_subscriber::fmt::init();
  tracing::info!("concat cover!");

  let args: Vec<String> = env::args().collect();
  if args.len() < 2 {
    tracing::error!("invalid arg. input arg to indicate path such as \"/202512\"");
    return;
  }

  let sub_dir: String = args[1].clone();

  tracing::info!("sub_dir {}", sub_dir);

  os_init();
  let dir_name = sub_dir;
  concat_cover(dir_name);


}