use rs_flow1000_parse::{base_lib::{concat_cover, os_init,} };
use std::env;


fn main() {
  println!("concat cover!");

  let args: Vec<String> = env::args().collect();
  if args.len() < 2 {
    println!("invalid arg. input arg to indicate path such as \"/202512\"");
    return;
  }

  let sub_dir: String = args[1].clone();

  println!("sub_dir {}", sub_dir);

  os_init();
  let dir_name = sub_dir;
  concat_cover(dir_name);


}