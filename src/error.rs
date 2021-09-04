#[derive(Debug)]
pub struct Error(String);

pub trait IntoError: std::fmt::Debug {}

impl <T: IntoError> From<T> for Error {
    fn from(other: T) -> Error {
        Error(format!("{}: {:?}", std::any::type_name::<T>(), other))
    }
}

impl IntoError for &str {}
impl IntoError for String {}
impl IntoError for std::io::Error {}
impl IntoError for mysql::Error {}
impl IntoError for resol_vbus::Error {}
impl IntoError for serial::Error {}
impl IntoError for toml::de::Error {}

pub type Result<T> = ::std::result::Result<T, Error>;
