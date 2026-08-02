use super::ui;

pub fn run() {
  println!("{}", ui::banner());
  println!(
    "{}{} {}{} {}{} {}{} {}{}",
    ui::strong("Y"),
    ui::subtle("AML"),
    ui::strong("E"),
    ui::subtle("diting and"),
    ui::strong("R"),
    ui::subtle("efactoring with"),
    ui::strong("B"),
    ui::subtle("etter"),
    ui::strong("A"),
    ui::subtle("ccuracy")
  );
}
