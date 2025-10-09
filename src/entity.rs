use serde_derive::Serialize;
#[derive(Serialize, Clone)]
#[derive(Debug)]
pub struct VideoEntity {
  pub id: u32,
  #[serde(rename = "videoFileName")]
  pub video_file_name: String,
  #[serde(rename = "coverFileName")]
  pub cover_file_name: String,
  #[serde(rename = "designationChar")]
  pub designation_char: String,
  #[serde(rename = "designationNum")]
  pub designation_num: String,
  #[serde(rename = "dirPath")]
  pub dir_path: String,
  #[serde(rename = "baseIndex")]
  pub base_index: u32,
  #[serde(rename = "rate")]
  pub rate: Option<u32>,
  #[serde(rename = "videoSize")]
  pub video_size: Option<u64>,
  #[serde(rename = "coverSize")]
  pub cover_size: Option<u64>,
  pub width: i32,
  pub height: i32,
  #[serde(rename = "frameRate")]
  pub frame_rate: i32,
  #[serde(rename = "videoFrameCount")]
  pub video_frame_count: i32,
  pub duration: i32,
  pub moved: Option<u8>,
  pub cfb: u8,
}

impl VideoEntity {
  pub fn new_by_file_name(
    id: u32,
    video_file_name: String,
    cover_file_name: String,
    dir_path: String,
    base_index:u32, 
  ) -> VideoEntity {
    VideoEntity {
      id,
      video_file_name,
      cover_file_name,
      designation_char: String::new(),
      designation_num: String::new(),
      dir_path,
      base_index,
      rate: Option::Some(0),
      video_size: Option::Some(0),
      height: 0,
      width: 0,
      frame_rate: 0,
      video_frame_count: 0,
      duration: 0,
      cover_size: Option::Some(0),
      moved: Option::Some(0),
      cfb: 0
    }
  }

  pub fn new_for_meta_info(
    id: u32,
    video_file_name: String,
    cover_file_name: String,
    dir_path: String,
    base_index: u32,
    video_size: Option<u64>,
    cover_size: Option<u64>,
    rate: Option<u32>,
    height: i32,
    width: i32,
    frame_rate: i32,
    video_frame_count: i32,
    duration: i32,
  ) -> VideoEntity {
    VideoEntity {
      id,
      video_file_name,
      cover_file_name,
      designation_char: String::new(),
      designation_num: String::new(),
      dir_path,
      base_index,
      video_size,
      cover_size,
      rate,
      height,
      width,
      frame_rate,
      video_frame_count,
      duration,
      moved: Option::Some(0),
      cfb: 0,
    }
  }

  pub fn new_for_base_info(
    id: u32,
    video_file_name: String,
    cover_file_name: String,
    video_size: Option<u64>,
    rate: Option<u32>,
    base_index: u32,
    dir_path: String,
  ) -> VideoEntity {
    VideoEntity {
      id,
      video_file_name,
      cover_file_name,
      designation_char: String::new(),
      designation_num: String::new(),
      dir_path,
      base_index,
      rate,
      video_size,
      height: 0,
      width: 0,
      frame_rate: 0,
      video_frame_count: 0,
      duration: 0,
      cover_size: Option::Some(0),
      moved: Option::Some(0),
      cfb: 0,
    }
  }

  pub fn new_by_for_duplicate_cover(
    id: u32,
    video_file_name: String,
    cover_file_name: String,
    dir_path: String,
    base_index: u32,
    designation_char: String,
    designation_num: String,
  ) -> VideoEntity {
    VideoEntity {
      id,
      video_file_name,
      cover_file_name,
      dir_path,
      base_index,
      designation_char,
      designation_num,
      rate: Option::Some(0),
      video_size: Option::Some(0),
      height: 0,
      width: 0,
      frame_rate: 0,
      video_frame_count: 0,
      duration: 0,
      cover_size: Option::Some(0),
      moved: Option::Some(0),
      cfb: 0,
    }
  }
}

#[derive(Serialize, Clone)]
pub struct DuplicateCoverEntity {
  pub count: u32,
  #[serde(rename = "coverFileName")]
  pub cover_file_name: String,
  #[serde(rename = "videoInfo")]
  pub video_info_list: Vec<VideoEntity>,
  #[serde(rename = "dirPath")]
  pub dir_path: String,
}

#[derive(Serialize, Clone)]
pub struct DuplicateEntity {
  pub count: u32,
  #[serde(rename = "designationChar")]
  pub designation_char: String,
  #[serde(rename = "designationNum")]
  pub designation_num: String,
  #[serde(rename = "videoInfo")]
  pub video_info_list: Vec<VideoEntity>,
}

#[derive(Serialize)]
pub struct MountConfig {
  pub id: u32,
  #[serde(rename = "baseDir")]
  pub dir_path: String,
  #[serde(rename = "urlPrefix")]
  pub url_prefix: String,
  #[serde(rename = "apiVersion")]
  pub api_version: u32,
}

#[derive(Serialize)]
pub struct TagEntity {
  pub id: u32,
  pub tag: String,
}
impl Clone for TagEntity {
  fn clone(&self) -> TagEntity {
    TagEntity {
      id: self.id,
      tag: self.tag.clone(),
    }
  }
}

// #[derive(Serialize)]
// pub struct VideoTagEntity {
//   pub id: u32,
//   #[serde(rename = "tagId")]
//   pub tag_id: u32,
//   #[serde(rename = "videoId")]
//   pub video_id: u32,
// }

#[derive(Serialize)]
pub struct StatisticEntity {
  #[serde(rename = "totalVideoSize")]
  pub total_video_size: u64,
  #[serde(rename = "totalCoverSize")]
  pub total_cover_size: u64,

  #[serde(rename = "deletedSize")]
  pub deleted_size: u64,
}
