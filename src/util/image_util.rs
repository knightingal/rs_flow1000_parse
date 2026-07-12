use std::{fs::File, io::{self, Error, Read, Seek}};
use std::io::ErrorKind;


pub fn parse_jpg_size(mut jpg: File, start: u64) -> io::Result<(u32, u32)> {

  jpg.seek(std::io::SeekFrom::Start(start))?;

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

pub fn parse_webp_size(mut webp: File, start: u64) -> io::Result<(u32, u32)> {
  webp.seek(std::io::SeekFrom::Start(start))?;

  let mut buf = [0u8; 16];
  webp.read_exact(&mut buf)?;
  
  let riff = String::from_utf8(Vec::from(&buf[0..4])).map_err(|_| Error::new(ErrorKind::Other, "unexpected header"))?;
  if riff != "RIFF" {
    return Err(
      Error::new(ErrorKind::Other, "unexpected header")
    );
  }
  
  let type_ = String::from_utf8(Vec::from(&buf[8..12])).map_err(|_| Error::new(ErrorKind::Other, "unexpected type"))?;
  if type_ != "WEBP" {
    return Err(
      Error::new(ErrorKind::Other, "unexpected type")
    );
  }
  
  let vp8 = String::from_utf8(Vec::from(&buf[12..16])).map_err(|_| Error::new(ErrorKind::Other, "unexpected vp8"))?;
  if vp8 != "VP8 " && vp8 != "VP8X" {
    return Err(
      Error::new(ErrorKind::Other, "unexpected vp8")
    );
  }
  
  if vp8 == "VP8 " {
    let mut buf = [0u8; 4];
    webp.read_exact(&mut buf)?;
    
    let mut buf = [0u8; 16];
    let read_len = webp.read(&mut buf)?;
    if read_len != 16 {
      return Err(Error::new(ErrorKind::Other, "read file failed, read header len: "));
    }
    
    let data6 = buf[6] as u32;
    let data7 = buf[7] as u32;
    let data8 = buf[8] as u32;
    let data9 = buf[9] as u32;
    
    let w = ((data7 << 8) | data6) & 0x3fff;
    let h = ((data9 << 8) | data8) & 0x3fff;
    
    Ok((h, w))
  } else {
    let mut buf = [0u8; 20];
    let read_len = webp.read(&mut buf)?;
    if read_len != 20 {
      return Err(Error::new(ErrorKind::Other, "read file failed, read header len: "));
    }
    
    let data12 = buf[8] as u32;
    let data13 = buf[9] as u32;
    let data14 = buf[10] as u32;
    let data15 = buf[11] as u32;
    let data16 = buf[12] as u32;
    let data17 = buf[13] as u32;
    
    let width = 1 + (data14 << 16 | data13 << 8 | data12);
    let height = 1 + (data17 << 16 | data16 << 8 | data15);
    
    Ok((height, width))
  }
}

pub fn parse_png_size(mut png: File, start: u64) -> io::Result<(u32, u32)> {
  png.seek(std::io::SeekFrom::Start(start))?;

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