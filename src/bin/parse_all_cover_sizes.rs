use rs_flow1000_parse::base_lib::{linux_init, parse_image_size_by_id, scan_all_by_id, update_cover_size_by_id};

fn main() {

  linux_init();

  let size_list = scan_all_by_id(|id| {
    println!("id: {}", id);
    let res = parse_image_size_by_id(id);
    if res.is_ok() {
      let (width, height) = res.unwrap();
      return Ok((id, width, height));
    } else {
      return Err(())
    }
  });

  size_list.iter().for_each(|res| {
    if res.is_ok() {
      let (id, width, height) = res.unwrap();
      println!("id{}, width:{}, heigth:{}",id, width, height);
      let update_ret = update_cover_size_by_id(id, width, height);
      if update_ret.is_err() {
        println!("fail to update id:{}, error:{}", id, update_ret.unwrap_err())
      }
    }
  });
}