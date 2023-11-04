#[derive(Debug)]
pub enum Error {
    Hafas(hafas_rs::Error),
    Timeout,
}

impl std::error::Error for Error {}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Error::Hafas(e) => write!(f, "Backend error: {}", e),
            Error::Timeout => write!(f, "Timed Out"),
        }
    }
}

impl From<hafas_rs::Error> for Error {
    fn from(e: hafas_rs::Error) -> Self {
        Self::Hafas(e)
    }
}
