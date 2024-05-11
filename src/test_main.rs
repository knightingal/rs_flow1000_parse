#[cfg(test)]
mod tests {
    use std::{fs::File, io::Write};

    use crate::parse_designation;

  #[test]
  fn it_works() {
    assert_eq!(2 + 2, 4);
  }

  #[test]
  fn test_parse_designation1() {
    let file_name = String::from("ipx-091");
    let designation = parse_designation(&file_name);
    assert_eq!(designation.char_len, 3);
    assert_eq!(designation.num_len, 3);
    assert_eq!(designation.char_part, "ipx");
    assert_eq!(designation.num_part, "091");
  }

  #[test]
  fn test_parse_designation2() {
    let file_name = String::from("ipx091");
    let designation = parse_designation(&file_name);
    assert_eq!(designation.char_len, 3);
    assert_eq!(designation.num_len, 3);
    assert_eq!(designation.char_part, "ipx");
    assert_eq!(designation.num_part, "091");
    
  }

  #[test]
  fn test_parse_designation3() {
    let file_name = String::from("中文ipx091");
    let designation = parse_designation(&file_name);
    assert_eq!(designation.char_len, 3);
    assert_eq!(designation.num_len, 3);
    assert_eq!(designation.char_part, "ipx");
    assert_eq!(designation.num_part, "091");
    
  }
  #[test]
  fn test_parse_designation4() {
    let file_name = String::from("中文@ipx091");
    let designation = parse_designation(&file_name);
    assert_eq!(designation.char_len, 3);
    assert_eq!(designation.num_len, 3);
    assert_eq!(designation.char_part, "ipx");
    assert_eq!(designation.num_part, "091");
  }

  #[test]
  fn test_parse_designation5() {
    let file_name = String::from("中文-ipx091");
    let designation = parse_designation(&file_name);
    assert_eq!(designation.char_len, 3);
    assert_eq!(designation.num_len, 3);
    assert_eq!(designation.char_part, "ipx");
    assert_eq!(designation.num_part, "091");
  }

  #[test]
  fn create_large_file() -> std::io::Result<()> {
    let mut f = File::create("testfile.hex")?;
    let mut i: u32 = 0;
    loop {
      if i <= 0xffff {
        let u0: u8= (i >> 24) as u8;
        let u1: u8= ((i >> 16) & 0xff) as u8;
        let u2: u8= ((i >> 8) & 0xff) as u8;
        let u3: u8= (i & 0xff) as u8;
        let buf = [u0, u1, u2, u3];
        let _ = f.write(&buf);
        if i & 0xffff == 0 {
          println!("{:08x}", i);
        }

      } else {
        break;
      }
      i = i + 1;
    }

    Ok(())
  }
}