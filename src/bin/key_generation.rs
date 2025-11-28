fn main() {
  let key_array: [u8; 32] = [0; 32];

  for i in 0..32 {
    print!("{:02x}", key_array[i]);
  }

  println!()
}