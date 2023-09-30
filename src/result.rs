pub type Result<T> = std::result::Result<T, Error>;

#[derive(Debug)]
pub enum Error {
    NEXRADError(nexrad::result::Error),
}

impl From<nexrad::result::Error> for Error {
    fn from(err: nexrad::result::Error) -> Error {
        Error::NEXRADError(err)
    }
}
