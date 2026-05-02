use yerba::Document;

pub fn parse(yaml: &str) -> Document {
  Document::parse(yaml).unwrap()
}
