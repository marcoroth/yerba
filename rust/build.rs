use std::env;
use std::path::PathBuf;

fn main() {
  let crate_dir = env::var("CARGO_MANIFEST_DIR").unwrap();
  let header_path = PathBuf::from(&crate_dir).join("../ext/yerba/include/yerba.h");

  if let Ok(bindings) = cbindgen::generate(&crate_dir) {
    bindings.write_to_file(&header_path);
  }
}
