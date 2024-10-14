use std::env;


fn main() {
  let link_type = env::var("LINK_TYPE").unwrap_or(String::from("real"));
  if link_type == "real" {
    println!("cargo::rustc-cfg=reallink");
  } else {
    println!("cargo::rustc-cfg=mocklink");
  }
  println!("cargo::rustc-check-cfg=cfg(reallink)");
  println!("cargo::rustc-check-cfg=cfg(mocklink)");
}