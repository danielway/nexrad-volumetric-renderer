pub type Result<T> = std::result::Result<T, Error>;

#[derive(Debug)]
pub enum Error {
    NEXRADError(nexrad::result::Error),
    IOError(std::io::Error),
    WindowError(three_d::WindowError),
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

impl From<three_d::WindowError> for Error {
    fn from(err: three_d::WindowError) -> Error {
        Error::WindowError(err)
    }
}
