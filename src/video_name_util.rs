use std::ffi::{c_char, c_void, CString};

use serde_derive::Serialize;

#[cfg(mocklink)]
use std::ptr::null_mut;

#[cfg(reallink)]
#[link(name = "frame_decode")]
extern "C" {
  fn video_meta_info(file_url: *const c_char) -> *mut VideoMetaInfo;
}

#[cfg(mocklink)]
fn video_meta_info(_: *const c_char) -> *mut VideoMetaInfo {
  return null_mut();
}

pub fn parse_video_meta_info(video_name: &String) -> VideoMetaInfo {
  let meta_info = unsafe {
    let video_name = CString::new(video_name.as_str()).unwrap();
    let p_meta_info = video_meta_info(video_name.as_ptr());
    let meta_info = (*p_meta_info).clone();
    libc::free(p_meta_info as *mut c_void);
    meta_info
  };
  meta_info
}

pub fn parse_video_cover(dir_list: &Vec<(String, u64)>) -> Vec<VideoCover> {
  let mut video_cover_list: Vec<VideoCover> = Vec::new();
  let mut video_file_name_list: Vec<(&String, u64)> = Vec::new();
  let mut img_file_name_list: Vec<&String> = Vec::new();
  for file_name in dir_list.iter() {
    if file_name.0.strip_suffix(".mp4").is_some() {
      video_file_name_list.push((&file_name.0, file_name.1));
    } else if file_name.0.strip_suffix(".jpg").is_some()
      || file_name.0.strip_suffix(".png").is_some()
    {
      img_file_name_list.push(&file_name.0);
    }
  }

  for video_file_name in video_file_name_list.into_iter() {
    let match_ret = video_match_to_cover(video_file_name, &img_file_name_list);
    if match_ret.is_ok() {
      video_cover_list.push(match_ret.unwrap());
    }
  }
  return video_cover_list;
}

fn video_match_to_cover(
  video_file_name: (&String, u64),
  img_file_name_list: &Vec<&String>,
) -> Result<VideoCover, ()> {
  let pure_name = parse_pure_name(video_file_name.0);
  let size = pure_name.chars().count();
  for i in 0..size {
    for j in 0..i + 1 {
      let ret = only_one_matched(&pure_name, img_file_name_list, j, j + size - i);

      if ret.is_ok() {
        return Ok(VideoCover {
          video_file_name: video_file_name.0.clone(),
          cover_file_name: ret.unwrap(),
          video_size: video_file_name.1,
        });
      }
    }
  }
  Err(())
}

fn only_one_matched(
  src: &String,
  img_file_name_list: &Vec<&String>,
  start: usize,
  end: usize,
) -> Result<String, bool> {
  let filted = img_file_name_list
    .into_iter()
    .filter(|img_file_name| sub_string_matched(src, &img_file_name, start, end));
  let mut matched_vec: Vec<&String> = Vec::new();
  for filted_it in filted {
    matched_vec.push(filted_it);
  }
  if matched_vec.len() != 1 {
    return Err(false);
  }

  return Ok(matched_vec[0].clone());
}

fn sub_string_matched(src: &String, target: &String, start: usize, end: usize) -> bool {
  println!("parse {} to {}", src, target);
  let target_len = target.chars().count();

  for i in 0..target_len {
    let mut target_iter = target.chars().skip(i);
    let mut src_iter = src.chars().skip(start);
    let mut matched = true;
    for _ in start..end {
      let src_iter_opt = src_iter.next();
      let target_iter_opt = target_iter.next();
      if src_iter_opt.is_none() || target_iter_opt.is_none() {
        matched = false;
        break;
      }

      if src_iter_opt.unwrap() != target_iter_opt.unwrap() {
        matched = false;
        break;
      }
    }
    if matched {
      return true;
    }
  }
  return false;
}

fn parse_pure_name(file_name: &String) -> String {
  return (file_name.strip_suffix(".mp4").unwrap()).to_string();
}

#[test]
fn video_match_to_cover_test() {
  let ret = video_match_to_cover(
    (&"1234567890.mp4".to_string(), 0),
    &vec![&"1234567890.jpg".to_string(), &"398434979.jpg".to_string()],
  )
  .unwrap();
  assert_eq!(ret.video_file_name, "1234567890.mp4".to_string());
  assert_eq!(ret.cover_file_name, "1234567890.jpg".to_string());

  let ret = video_match_to_cover(
    (&"23456789.mp4".to_string(), 0),
    &vec![&"1234567890.jpg".to_string(), &"398434979.jpg".to_string()],
  )
  .unwrap();
  assert_eq!(ret.video_file_name, "23456789.mp4".to_string());
  assert_eq!(ret.cover_file_name, "1234567890.jpg".to_string());

  let ret = video_match_to_cover(
    (&"234567891.mp4".to_string(), 0),
    &vec![&"1234567890.jpg".to_string(), &"398434979.jpg".to_string()],
  )
  .unwrap();
  assert_eq!(ret.video_file_name, "234567891.mp4".to_string());
  assert_eq!(ret.cover_file_name, "1234567890.jpg".to_string());

  let ret = video_match_to_cover(
    (&"234567891.mp4".to_string(), 0),
    &vec![
      &"1234567890.jpg".to_string(),
      &"1234567892.jpg".to_string(),
      &"398434979.jpg".to_string(),
    ],
  );
  assert_eq!(ret.is_err(), true);
}

#[test]
fn only_one_matched_test() {
  // let matched = only_one_matched(&"1234567890".to_string(), &vec![&"1234567890.jpg".to_string(), &"398434979.jpg".to_string()]).unwrap();
  // assert_eq!(matched, "1234567890.jpg".to_string());
}

#[test]
fn test_sub_string_matched() {
  let matched = sub_string_matched(
    &String::from("中国江西南京"),
    &String::from("江苏南京鼓楼"),
    2,
    4,
  );
  assert_eq!(matched, false);

  let matched = sub_string_matched(
    &String::from("中国江苏南京"),
    &String::from("江苏南京鼓楼"),
    2,
    4,
  );
  assert_eq!(matched, true);

  let matched = sub_string_matched(
    &String::from("中国江苏南京"),
    &String::from("江苏南京鼓楼"),
    2,
    6,
  );
  assert_eq!(matched, true);

  let matched = sub_string_matched(
    &String::from("中国江苏南京"),
    &String::from("江苏南京鼓楼"),
    1,
    6,
  );
  assert_eq!(matched, false);
}

#[derive(Debug, Serialize)]
pub struct VideoCover {
  pub video_file_name: String,
  pub cover_file_name: String,
  pub video_size: u64,
}

#[repr(C)]
#[derive(Serialize, Clone)]
pub struct VideoMetaInfo {
  pub width: i32,
  pub height: i32,
  #[serde(rename = "frameRate")]
  pub frame_rate: i32,
  #[serde(rename = "videoFrameCount")]
  pub video_frame_count: i32,
  pub duratoin: i32,
  pub size: u64,
}
