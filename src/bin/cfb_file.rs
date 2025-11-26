use std::ffi::c_char;

use rs_flow1000_parse::base_lib::init_key;

#[cfg(reallink)]
#[link(name = "cfb_decode")]
extern "C" {

  fn key_expansion(key: *const u8, w: *mut u32);
  fn cfb_file_streaming_v2(
    w: *const u32,
    iv: *const u8,
    input_filename: *const c_char,
    output_filename: *const c_char,
  ) -> i32;
}

fn main() {
  init_key();

  unsafe {
    if cfg!(reallink) {
      let iv = "2021000120210001";
      let ret = cfb_file_streaming_v2(
        0 as *const u32,
        iv.as_ptr(),
        "/home/knightingal/demo_video.mp4\0".as_ptr() as *const c_char,
        "/home/knightingal/rust_cfb.mp4.bin\0".as_ptr() as *const c_char,
      );
      println!("cfb_file_streaming_v2 ret: {}", ret);
    }
  }
}