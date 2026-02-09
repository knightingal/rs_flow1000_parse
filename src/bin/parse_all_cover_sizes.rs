use rs_flow1000_parse::base_lib::{linux_init, parse_image_size_by_id, scan_all_by_id, update_cover_size_by_id};

fn main() {

  linux_init();

  scan_all_by_id(|id| {
    println!("id: {}", id);
    let res = parse_image_size_by_id(id);
    if res.is_ok() {
      let (width, height) = res.unwrap();
      println!("width:{}, heigth:{}", width, height);
      let update_ret = update_cover_size_by_id(id, width, height);
      if update_ret.is_err() {
        println!("fail to update id:{}, error:{}", id, update_ret.unwrap_err())
      }
    } else {
      println!("parse {} failed", id);
    }
  });
}