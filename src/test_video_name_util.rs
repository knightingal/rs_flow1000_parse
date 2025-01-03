#[cfg(test)]
mod tests {
  use crate::video_name_util::{sub_string_matched, video_match_to_cover};

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
}
