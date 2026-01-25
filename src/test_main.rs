#[cfg(test)]
mod tests {
  use std::{ffi::{CString, c_void}, fs::File, io::{Read, Seek, SeekFrom, Write}, };


use crate::{ handles::snapshot, util::image_util::{parse_jpg_size, parse_png_size} };

  #[test]
  fn move_cover_test() {
    // move_cover();

    let video_name = CString::new("/home/knightingal/demo_video.mp4").unwrap();
    let snapshot_st = snapshot(video_name, 10);

    unsafe { libc::free(snapshot_st.buff as *mut c_void) };
    println!("snapshot_st len:{}", snapshot_st.buff_len)
  }

  #[test]
  fn create_large_file() -> std::io::Result<()> {
    let mut f = File::create("testfile.hex")?;
    let mut i: u32 = 0;
    loop {
      if i <= 0xffff {
        let u0: u8= ( i >> 24)         as u8;
        let u1: u8= ((i >> 16) & 0xff) as u8;
        let u2: u8= ((i >> 08) & 0xff) as u8;
        let u3: u8= ( i        & 0xff) as u8;
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

  #[test]
  fn seek_file() -> std::io::Result<()> {
    let mut f = File::open("testfile.bin")?;
    let pos : SeekFrom = SeekFrom::Start(0x3ffffffc);
    let _ = f.seek(pos);
    let mut buf:[u8; 4] = [0,0,0,0];
    let _ = f.read(& mut buf);
    println!("{:02x},{:02x},{:02x},{:02x},", buf[0], buf[1], buf[2], buf[3]);
    Ok(())
  }

  #[test]
  fn test_parse_png_size() {
    let png = File::open("/home/knightingal/Pictures/Screenshots/ScreenshotFrom2026-01-2421-51-29.png").unwrap();
    let (width, heigth) = parse_png_size(png, 0).unwrap();
    println!("width:{}, height:{}", width, heigth);
  }

  #[test]
  fn test_parse_jpg_size() {
    let jpg = File::open("/home/knightingal/Pictures/llqdfm.jpg").unwrap();
    let (width, heigth) = parse_jpg_size(jpg, 0).unwrap();
    println!("width:{}, height:{}", width, heigth);
  }

}