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

  #[test]
  fn test_parse_designation9() {
    let file_name = String::from("@江南@jnty4588.com-DVAJ-642_2K.mp4");
    let designation = parse_designation(&file_name);
    assert_eq!(designation.char_final.unwrap(), "DVAJ");
    assert_eq!(designation.num_final.unwrap(), "642");
  }

  #[test]
  fn test_parse_designation10() {
    let file_name = String::from("[gg5.co]WAAA-345.mp4");
    let designation = parse_designation(&file_name);
    assert_eq!(designation.char_final.unwrap(), "WAAA");
    assert_eq!(designation.num_final.unwrap(), "345");
  }

  #[test]
  fn test_parse_designation11() {
    let file_name = String::from("aavv39.xyz@IPX888C.mp4");
    let designation = parse_designation(&file_name);
    assert_eq!(designation.char_final.unwrap(), "IPX");
    assert_eq!(designation.num_final.unwrap(), "888");
  }

  #[test]
  fn test_parse_designation12() {
    let file_name = String::from("gc2048.com-中文字幕 ，一，些中文SSNI897【水印】.mp4");
    let designation = parse_designation(&file_name);
    assert_eq!(designation.char_final.unwrap(), "SSNI");
    assert_eq!(designation.num_final.unwrap(), "897");
  }

  #[test]
  fn test_parse_designation13() {
    let file_name = String::from("SSIS-656~nyap2p.com.mp4");
    let designation = parse_designation(&file_name);
    assert_eq!(designation.char_final.unwrap(), "SSIS");
    assert_eq!(designation.num_final.unwrap(), "656");
  }
}