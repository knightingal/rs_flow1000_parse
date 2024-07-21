use std::borrow::{Borrow, BorrowMut};

fn video_match_to_cover(video_file_name: String, img_file_name_list: &Vec<String>) -> Result<VideoCover, ()> {

  let pure_name = parse_pure_name(&video_file_name);
  let size = pure_name.len();
  for i in 0..size {
    for j in 0..i+1 {
      let sub1:String = pure_name.split_at(j + size -i).0.into();
      let sub2:String = sub1.split_at(j).1.into();
      let ret = only_one_matched(sub2, img_file_name_list);

      if ret.is_ok() {
        return Ok(VideoCover { video_file_name: video_file_name, cover_file_name: ret.unwrap() })
      }
    }
  }
  Err(())
}

fn only_one_matched(src: String, img_file_name_list: &Vec<String>) -> Result<String, bool> {
  let mut filted = img_file_name_list.into_iter().filter(|img_file_name| {
    img_file_name.contains(&src)
  });
  if filted.borrow_mut().count() > 1 {
    return Err(false);
  }

  return Ok(filted.next().unwrap().clone());

}

fn parse_pure_name(file_name: &String) -> String {
  return String::from(file_name.split('.').next().unwrap());

}

struct VideoCover {
  video_file_name: String,
  cover_file_name: String,
}