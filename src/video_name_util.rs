use std::borrow::{Borrow, BorrowMut};


fn video_match_to_cover(video_file_name: String, img_file_name_list: &Vec<String>) -> Result<VideoCover, ()> {
  let pure_name = parse_pure_name(&video_file_name);
  let size = pure_name.len();
  for i in 0..size {
    for j in 0..i + 1 {
      let sub1:String = pure_name.split_at(j + size - i).0.into();
      let sub2:String = sub1.split_at(j).1.into();
      println!("{}", sub2);
      let ret = only_one_matched(sub2, img_file_name_list);

      if ret.is_ok() {
        return Ok(VideoCover { video_file_name: video_file_name, cover_file_name: ret.unwrap() })
      }
    }
  }
  Err(())
}

fn only_one_matched(src: String, img_file_name_list: &Vec<String>) -> Result<String, bool> {
  let filted = img_file_name_list.into_iter().filter(|img_file_name| {
    parse_pure_name(img_file_name).contains(&src)
  });
  let mut matched_vec: Vec<&String> = Vec::new();
  for filted_it in filted {
    matched_vec.push(filted_it);
  }
  if matched_vec.len() != 1 {
    return Err(false);
  }

  return Ok(matched_vec[0].clone());
}

fn parse_pure_name(file_name: &String) -> String {
  return String::from(file_name.split('.').next().unwrap());
}

#[test]
fn video_match_to_cover_test() {
  let ret = video_match_to_cover("1234567890.mp4".to_string(), &vec!["1234567890.jpg".to_string(), "398434979.jpg".to_string()]).unwrap();
  assert_eq!(ret.video_file_name, "1234567890.mp4".to_string());
  assert_eq!(ret.cover_file_name, "1234567890.jpg".to_string());
}

#[test]
fn only_one_matched_test() {
  let matched = only_one_matched("1234567890".to_string(), &vec!["1234567890.jpg".to_string(), "398434979.jpg".to_string()]).unwrap();
  assert_eq!(matched, "1234567890.jpg".to_string());
}

struct VideoCover {
  video_file_name: String,
  cover_file_name: String,
}