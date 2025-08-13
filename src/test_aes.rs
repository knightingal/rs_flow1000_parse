#[cfg(test)]
mod tests {
  use aes::cipher::{KeyIvInit, StreamCipher, StreamCipherSeek};
  use hex_literal::hex;

  type Aes128Ctr64LE = ctr::Ctr64LE<aes::Aes128>;

#[cfg(reallink)]
#[link(name = "cfb_decode")]
extern "C" {
  fn cfb_v2(w: *const u32, iv: *const u8, input_buf: *const u8, output: *mut u8, len: usize);
  #[allow(dead_code)]
  fn inv_cfb_v2(w: *const u32, iv: *const u8, input_buf: *const u8, output: *mut u8, len: usize);
  fn key_expansion(key: *const u8, w: *mut u32);
}

  #[test]
  fn cfb_extern_test() {
    unsafe {
      if cfg!(reallink) {
        let key = "passwordpasswordpasswordpassword";
        let iv = "2021000120210001";
        let input_data = "0123456789abcdef0123456789abcdef";
        let mut w: [u32; 60] = [0; 60];
        key_expansion(key.as_ptr(), w.as_mut_ptr());
        println!("key_expansion: {:?}", w);
        let mut output = [0u8; 32];
        cfb_v2(w.as_ptr(), iv.as_ptr(), input_data.as_ptr(), output.as_mut_ptr(), input_data.len());
        println!("output: {:?}", output);
      }
    }
  }


  #[test]
  fn ctr_test() {
    let key = [0x42; 16];
    let iv = [0x24; 16];
    let plaintext = *b"hello world! this is my plaintext.";
    let ciphertext = hex!(
        "3357121ebb5a29468bd861467596ce3da59bdee42dcc0614dea955368d8a5dc0cad4"
    );
    
    // encrypt in-place
    let mut buf = plaintext.to_vec();
    let mut cipher = Aes128Ctr64LE::new(&key.into(), &iv.into());
    cipher.apply_keystream(&mut buf);
    assert_eq!(buf[..], ciphertext[..]);
    
    // CTR mode can be used with streaming messages
    let mut cipher = Aes128Ctr64LE::new(&key.into(), &iv.into());
    for chunk in buf.chunks_mut(3) {
        cipher.apply_keystream(chunk);
    }
    assert_eq!(buf[..], plaintext[..]);
    
    // CTR mode supports seeking. The parameter is zero-based _bytes_ counter (not _blocks_).
    cipher.seek(0u32);
    
    // encrypt/decrypt from buffer to buffer
    // buffer length must be equal to input length
    let mut buf1 = [0u8; 34];
    cipher
        .apply_keystream_b2b(&plaintext, &mut buf1)
        .unwrap();
    assert_eq!(buf1[..], ciphertext[..]);
    
    let mut buf2 = [0u8; 34];
    cipher.seek(0u32);
    cipher.apply_keystream_b2b(&buf1, &mut buf2).unwrap();
    assert_eq!(buf2[..], plaintext[..]);
  }

  #[test]
  fn ctr_test1() {

    let key = [0u8; 16];
    let iv = [0u8; 16];

    let plainbin = [1u8; 256];
    let (_, plainsub) = plainbin.split_at(128);
    // let plainbuf = plainbin.to_vec();

    let mut cipher = Aes128Ctr64LE::new(&key.into(), &iv.into());

    cipher.seek(0u32);

    let mut encr_buf = [0u8; 256];
    let mut recu_buf = [0u8; 128];

    cipher
        .apply_keystream_b2b(&plainbin, &mut encr_buf)
        .unwrap();
    println!("{}", encr_buf[0]);
    let (_, sub) = encr_buf.split_at(128);

    cipher.seek(128);
    cipher
        .apply_keystream_b2b(sub, &mut recu_buf)
        .unwrap();
    println!("{}", recu_buf[0]);
    assert_eq!(plainsub[..], recu_buf[..])
  }

  #[test]
  fn ctr_test2() {

    let key = [1u8; 16];
    let iv = [8u8; 16];

    let mut plainbin = Vec::new();
    let mut i = 0u8;
    loop {
      if i < 128 {
        plainbin.push(i);
        i = i + 1;

      } else {
        break;
      }
    }

    let (_, plainsub) = plainbin.split_at(64);
    // let plainbuf = plainbin.to_vec();

    let mut cipher = Aes128Ctr64LE::new(&key.into(), &iv.into());

    cipher.seek(0u32);

    let mut encr_buf = [0u8; 128];
    let mut recu_buf = [0u8; 64];

    cipher
        .apply_keystream_b2b(&plainbin, &mut encr_buf)
        .unwrap();
    println!("{}", encr_buf[0]);
    let (_, sub) = encr_buf.split_at(64);

    cipher.seek(64);
    cipher
        .apply_keystream_b2b(sub, &mut recu_buf)
        .unwrap();
    println!("{}", recu_buf[0]);
    assert_eq!(plainsub[..], recu_buf[..])
  }

  #[test]
  fn ctr_test3() {

    let key = [0u8; 16];
    let iv = [0u8; 16];

    let mut plainbin = Vec::new();
    let mut i = 0u8;
    loop {
      if i < 128 {
        plainbin.push(i);
        i = i + 1;

      } else {
        break;
      }
    }

    let (_, plainsub) = plainbin.split_at(63);
    // let plainbuf = plainbin.to_vec();

    let mut cipher = Aes128Ctr64LE::new(&key.into(), &iv.into());

    cipher.seek(0u32);

    let mut encr_buf = [0u8; 128];
    let mut recu_buf = [0u8; 65];

    cipher
        .apply_keystream_b2b(&plainbin, &mut encr_buf)
        .unwrap();
    println!("{}", encr_buf[0]);
    let (_, sub) = encr_buf.split_at(63);

    cipher.seek(63);
    cipher
        .apply_keystream_b2b(sub, &mut recu_buf)
        .unwrap();
    println!("{}", recu_buf[0]);
    assert_eq!(plainsub[..], recu_buf[..])
  }
}