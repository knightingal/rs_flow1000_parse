use rs_flow1000_parse::{base_lib::{concat_cover, os_init,} };


fn main() {
  println!("concat cover!");

  os_init();
  let dir_name = String::from("/202604");
  concat_cover(dir_name);


}