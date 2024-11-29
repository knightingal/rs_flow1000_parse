#[cfg(test)]
mod tests {
  use std::{fs::File, io::{Read, Seek, SeekFrom, Write}};

use crate::handles::move_cover;

  #[test]
  fn it_works() {
    assert_eq!(2 + 2, 4);
  }

  #[test]
  fn move_cover_test() {
    move_cover();
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
}