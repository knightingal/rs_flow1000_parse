use rs_flow1000_parse::base_lib::{linux_init, parse_image_size_by_id, scan_all_by_id};

fn main() {

  linux_init();

  scan_all_by_id(|id| {
    println!("id: {}", id);
    let res = parse_image_size_by_id(id);
    if res.is_ok() {
      let (width, heigth) = res.unwrap();
      println!("width:{}, heigth:{}", width, heigth);
    } else {
      println!("parse {} failed", id);
    }
  });
}