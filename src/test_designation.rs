#[cfg(test)]
mod tests {

  use crate::parse_designation;
  #[test]
  fn test_parse_designation1() {
    let file_name = String::from("ipx-091");
    let designation = parse_designation(&file_name);
    assert_eq!(designation.char_len, 3);
    assert_eq!(designation.num_len, 3);
    assert_eq!(designation.char_part, "ipx");
    assert_eq!(designation.num_part, "091");
    assert_eq!(designation.char_final.unwrap(), "ipx");
    assert_eq!(designation.num_final.unwrap(), "091");
  }

  #[test]
  fn test_parse_designation2() {
    let file_name = String::from("ipx091");
    let designation = parse_designation(&file_name);
    assert_eq!(designation.char_len, 3);
    assert_eq!(designation.num_len, 3);
    assert_eq!(designation.char_part, "ipx");
    assert_eq!(designation.num_part, "091");
    assert_eq!(designation.char_final.unwrap(), "ipx");
    assert_eq!(designation.num_final.unwrap(), "091");
  }

  #[test]
  fn test_parse_designation3() {
    let file_name = String::from("中文ipx091");
    let designation = parse_designation(&file_name);
    assert_eq!(designation.char_len, 3);
    assert_eq!(designation.num_len, 3);
    assert_eq!(designation.char_part, "ipx");
    assert_eq!(designation.num_part, "091");
    assert_eq!(designation.char_final.unwrap(), "ipx");
    assert_eq!(designation.num_final.unwrap(), "091");
  }
  #[test]
  fn test_parse_designation4() {
    let file_name = String::from("中文@ipx091");
    let designation = parse_designation(&file_name);
    assert_eq!(designation.char_len, 3);
    assert_eq!(designation.num_len, 3);
    assert_eq!(designation.char_part, "ipx");
    assert_eq!(designation.num_part, "091");
    assert_eq!(designation.char_final.unwrap(), "ipx");
    assert_eq!(designation.num_final.unwrap(), "091");
  }

  #[test]
  fn test_parse_designation5() {
    let file_name = String::from("中文-ipx091");
    let designation = parse_designation(&file_name);
    assert_eq!(designation.char_len, 3);
    assert_eq!(designation.num_len, 3);
    assert_eq!(designation.char_part, "ipx");
    assert_eq!(designation.num_part, "091");
    assert_eq!(designation.char_final.unwrap(), "ipx");
    assert_eq!(designation.num_final.unwrap(), "091");
  }

  #[test]
  fn test_parse_designation6() {
    let file_name = String::from("ipx-091a");
    let designation = parse_designation(&file_name);
    assert_eq!(designation.char_final.unwrap(), "ipx");
    assert_eq!(designation.num_final.unwrap(), "091");
  }

  #[test]
  fn test_parse_designation7() {
    let file_name = String::from("ipx-091@");
    let designation = parse_designation(&file_name);
    assert_eq!(designation.char_final.unwrap(), "ipx");
    assert_eq!(designation.num_final.unwrap(), "091");
  }

  #[test]
  fn test_parse_designation8() {
    let file_name = String::from("ipx-091.mp4");
    let designation = parse_designation(&file_name);
    assert_eq!(designation.char_final.unwrap(), "ipx");
    assert_eq!(designation.num_final.unwrap(), "091");
  }

}