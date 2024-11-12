use serde_derive::Serialize;
#[derive(Serialize, Clone)]
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
  pub width: i32,
  pub height: i32,
  #[serde(rename = "frameRate")]
  pub frame_rate: i32,
  #[serde(rename = "videoFrameCount")]
  pub video_frame_count: i32,
  pub duration: i32,
}

#[derive(Serialize, Clone)]
pub struct DuplicateCoverEntity {
  pub count: u32,
  #[serde(rename = "coverFileName")]
  pub cover_file_name: String,
  #[serde(rename = "videoInfo")]
  pub video_info_list: Vec<VideoEntity>,
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

#[derive(Serialize)]
pub struct VideoTagEntity {
  pub id: u32,
  #[serde(rename = "tagId")]
  pub tag_id: u32,
  #[serde(rename = "videoId")]
  pub video_id: u32,
}
