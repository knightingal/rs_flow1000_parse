use std::{io::Error, fs::File, io::{self, Read}};
use std::io::ErrorKind;


pub fn parse_jpg_size(mut jpg: File) -> io::Result<(u32, u32)> {
  let mut buf = [0u8; 1];
  jpg.read_exact(&mut buf)?;
  let b1 = buf[0];

  let mut buf = [0u8; 1];
  jpg.read_exact(&mut buf)?;
  let b2 = buf[0];

  if b1 != 0xff || b2 != 0xd8 {
    return Err(
      Error::new(ErrorKind::Other, "not find jpg header")
    );
  }

  loop {
    let mut buf = [0u8; 1];
    jpg.read_exact(&mut buf)?;
    let ff = buf[0];
    if ff != 0xff {
      return Err(
        Error::new(ErrorKind::Other, "expected 0xff marker")
      );
    }

    let mut buf = [0u8; 1];
    jpg.read_exact(&mut buf)?;
    let mut marker_type = buf[0];

    while marker_type == 0xff {
      let mut buf = [0u8; 1];
      jpg.read_exact(&mut buf)?;
      marker_type = buf[0];
    }
    
    let is_sof = marker_type >= 0xc0 && marker_type <= 0xcf 
        && marker_type != 0xc4
        && marker_type != 0xc8
        && marker_type != 0xcc;

    if is_sof {
      let mut buf = [0u8; 1];
      jpg.read_exact(&mut buf)?;
      let mut buf = [0u8; 1];
      jpg.read_exact(&mut buf)?;
      let mut buf = [0u8; 1];
      jpg.read_exact(&mut buf)?;

      let mut buf = [0u8; 2];
      jpg.read_exact(&mut buf)?;
      let height = (buf[0] as u32) << 8 | (buf[1] as u32);

      let mut buf = [0u8; 2];
      jpg.read_exact(&mut buf)?;
      let width = (buf[0] as u32) << 8 | (buf[1] as u32);
      return Ok((width, height));
    } else if marker_type == 0xd9 {
      return Err(
        Error::new(ErrorKind::Other, "Reached end of image without finding size")
      );
    } else if marker_type == 0xd8 {
      continue;
    } else if marker_type >= 0xd0 && marker_type <= 0xd7 {
      continue;
    } else if marker_type == 0x01 || marker_type == 0x00 {
      continue;
    } else {
      let mut buf = [0u8; 2];
      jpg.read_exact(&mut buf)?;
      let length = (buf[0] as u32) << 8 | (buf[1] as u32);
      let mut buf = vec![0; length as usize - 2];
      jpg.read_exact(&mut buf)?;
    }
  }
}

pub fn parse_png_size(mut png: File) -> io::Result<(u32, u32)> {
  let mut buf = [0u8; 8];
  png.read_exact(&mut buf)?;

  let mut buf = [0u8; 4];
  png.read_exact(&mut buf)?;

  let mut buf = [0u8; 4];
  png.read_exact(&mut buf)?;


  let chunk_type: String = String::from_utf8(Vec::from(buf)).unwrap();
  if !chunk_type.eq("IHDR") {
    return Err(
      Error::new(ErrorKind::Other, "not find IHDR")
    );
  }

  let mut buf = [0u8; 4];
  png.read_exact(&mut buf)?;
  let width: u32 = 
    (buf[0] as u32) << 24 |
    (buf[1] as u32) << 16 |
    (buf[2] as u32) <<  8 |
    (buf[3] as u32) <<  0 ;

  let mut buf = [0u8; 4];
  png.read_exact(&mut buf)?;
  let height: u32 = 
    (buf[0] as u32) << 24 |
    (buf[1] as u32) << 16 |
    (buf[2] as u32) <<  8 |
    (buf[3] as u32) <<  0 ;

  Ok((width, height))
}