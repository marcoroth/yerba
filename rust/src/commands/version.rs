use super::color::*;

pub fn run() {
  println!("🧉 {BOLD}yerba{RESET} {DIM}v{}{RESET}", yerba::version());
  println!(
    "   {BOLD}Y{RESET}{DIM}AML{RESET} {BOLD}E{RESET}{DIM}diting and{RESET} {BOLD}R{RESET}{DIM}efactoring with{RESET} {BOLD}B{RESET}{DIM}etter{RESET} {BOLD}A{RESET}{DIM}ccuracy{RESET}"
  );
}
