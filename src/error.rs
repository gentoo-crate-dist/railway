#[derive(Debug)]
pub enum Error {
    Hafas(rcore::Error<Box<dyn std::error::Error + Send>, Box<dyn std::error::Error + Send>>),
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

impl<R: std::error::Error + Send + 'static, P: std::error::Error + Send + 'static>
    From<rcore::Error<R, P>> for Error
{
    fn from(e: rcore::Error<R, P>) -> Self {
        match e {
            rcore::Error::Request(r) => Self::Hafas(rcore::Error::Request(
                Box::new(r) as Box<dyn std::error::Error + Send>
            )),
            rcore::Error::Provider(r) => Self::Hafas(rcore::Error::Provider(
                Box::new(r) as Box<dyn std::error::Error + Send>
            )),
        }
    }
}
