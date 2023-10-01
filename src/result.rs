pub type Result<T> = std::result::Result<T, Error>;

#[derive(Debug)]
pub enum Error {
    NEXRADError(nexrad::result::Error),
    IOError(std::io::Error),
}

impl From<nexrad::result::Error> for Error {
    fn from(err: nexrad::result::Error) -> Error {
        Error::NEXRADError(err)
    }
}

impl From<std::io::Error> for Error {
    fn from(err: std::io::Error) -> Error {
        Error::IOError(err)
    }
}
