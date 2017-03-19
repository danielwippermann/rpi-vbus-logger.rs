#[derive(Debug)]
pub enum Error {
    AppError(String),
    IoError(::std::io::Error),
    MysqlError(::mysql::Error),
    SerialError(::serial::Error),
    TomlDeError(::toml::de::Error),
}


impl From<String> for Error {
    fn from(inner: String) -> Error {
        Error::AppError(inner)
    }
}


impl From<::std::io::Error> for Error {
    fn from(inner: ::std::io::Error) -> Error {
        Error::IoError(inner)
    }
}


impl From<::mysql::Error> for Error {
    fn from(inner: ::mysql::Error) -> Error {
        Error::MysqlError(inner)
    }
}


impl From<::serial::Error> for Error {
    fn from(inner: ::serial::Error) -> Error {
        Error::SerialError(inner)
    }
}


impl From<::toml::de::Error> for Error {
    fn from(inner: ::toml::de::Error) -> Error {
        Error::TomlDeError(inner)
    }
}


pub type Result<T> = ::std::result::Result<T, Error>;
