use rs_flow1000_parse::base_lib::{get_sqlite_connection, os_init, parse_image_size_by_id, scan_all_by_id};
use rusqlite::named_params;

fn main() {

  os_init();

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

pub fn update_cover_size_by_id(id: u32, width: u32, height: u32) -> rusqlite::Result<()> {
  let sqlite_conn = get_sqlite_connection();
  let mut stmt = sqlite_conn.prepare("update video_info set cover_width = :width, cover_height = :height where id = :id").unwrap();
  stmt.execute(named_params! {":width": width, ":height": height, ":id": id})?;
  Ok(())
}