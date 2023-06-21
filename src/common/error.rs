#[derive(Debug)]
pub enum Error {
    Length(&'static str),
    InvalidValue(&'static str),
    Unknown,
}
