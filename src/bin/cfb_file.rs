use std::ffi::c_char;

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

  unsafe {
    if cfg!(reallink) {
      let key = "passwordpasswordpasswordpassword";
      let iv = "2021000120210001";
      let mut w: [u32; 60] = [0; 60];
      key_expansion(key.as_ptr(), w.as_mut_ptr());
      println!("key_expansion: {:?}", w);
      let ret = cfb_file_streaming_v2(
        w.as_ptr(),
        iv.as_ptr(),
        "/home/knightingal/demo_video.mp4\0".as_ptr() as *const c_char,
        "/home/knightingal/rust_cfb.mp4.bin\0".as_ptr() as *const c_char,
      );
      println!("cfb_file_streaming_v2 ret: {}", ret);
    }
  }
}