use super::color::*;

pub fn run() {
  let g = GREEN;
  let b = BOLD;
  let d = DIM;
  let i = "\x1b[3m"; // italic
  let y = YELLOW;
  let r = RESET;
  let hr = format!("    {d}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}{r}");

  println!();
  println!("    {b}{g}\u{1f9c9} YERBA MATE{r}");
  println!();
  println!("    {d}From the Guarani people of South America to the world.{r}");
  println!("    {d}Brewed from the leaves of{r} {i}Ilex paraguariensis{r}{d}, shared{r}");
  println!("    {d}in a hollowed calabaza, sipped through a metal bombilla.{r}");
  println!("    {d}One cup, many rounds.{r}");
  println!();
  println!("{hr}");
  println!();
  println!("    {b}{g}yerba{r} {d}/\u{02c8}\u{0292}\u{025b}\u{027e}.ba/{r} {y}noun{r}");
  println!("    The dried leaves of {i}Ilex paraguariensis{r}, used to brew");
  println!("    mate. From Guarani {i}ka'a{r}, meaning \"herb\".");
  println!();
  println!("    {b}{g}mate{r} {d}/\u{02c8}ma.te/{r} {y}noun{r}");
  println!("    A traditional South American caffeine-rich infusion.");
  println!("    Also the hollowed calabaza (gourd) from which it is drunk.");
  println!();
  println!("    {b}{g}bombilla{r} {d}/bom\u{02c8}bi.\u{0292}a/{r} {y}noun{r}");
  println!("    A metal straw with a filtered tip at the bottom, used");
  println!("    to sip mate without swallowing the leaves.");
  println!();
  println!("    {b}{g}cebador{r} {d}/se.ba\u{02c8}\u{00f0}o\u{027e}/{r} {y}noun{r}");
  println!("    The person who prepares and serves mate to the group.");
  println!("    A role of care, not hierarchy.");
  println!();
  println!("    {b}{g}ronda{r} {d}/\u{02c8}ron.da/{r} {y}noun{r}");
  println!("    The circle of people sharing mate. The cup passes");
  println!("    from hand to hand until it returns to the cebador.");
  println!();
  println!("    {b}{g}cebar{r} {d}/se\u{02c8}ba\u{027e}/{r} {y}verb{r}");
  println!("    To pour hot water over the yerba and serve a round.");
  println!("    The act of preparing each individual serving.");
  println!();
  println!("    {b}{g}aprontar{r} {d}/ap\u{027e}on\u{02c8}ta\u{027e}/{r} {y}verb{r}");
  println!("    To set up mate before the first pour: arranging the");
  println!("    yerba, heating the water, positioning the bombilla.");
  println!();
  println!("    {b}{g}ensillar{r} {d}/ensi\u{02c8}\u{0292}a\u{027e}/{r} {y}verb{r}");
  println!("    To replace spent yerba with fresh leaves mid-session,");
  println!("    extending the life of the mate.");
  println!();
  println!("{hr}");
}
