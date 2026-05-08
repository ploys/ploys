pub mod access_token;
pub mod device_code;

use std::convert::Infallible;
use std::error::Error;
use std::fmt::Debug;

use reqwest::blocking::Client as HttpClient;

use super::Credentials;

pub trait Authenticate: Debug + Send + Sync + 'static {
    type Error: Error + Send + Sync + 'static;

    fn authenticate(
        &self,
        credentials: &mut Option<Credentials>,
        http_client: &HttpClient,
    ) -> Result<(), Self::Error>;
}

impl Authenticate for () {
    type Error = Infallible;

    fn authenticate(&self, _: &mut Option<Credentials>, _: &HttpClient) -> Result<(), Self::Error> {
        Ok(())
    }
}

pub(super) trait DynAuthenticate: Debug {
    fn dyn_authenticate(
        &self,
        credentials: &mut Option<Credentials>,
        http_client: &HttpClient,
    ) -> Result<(), Box<dyn Error + Send + Sync>>;
}

impl<T> DynAuthenticate for T
where
    T: Authenticate,
{
    fn dyn_authenticate(
        &self,
        credentials: &mut Option<Credentials>,
        http_client: &HttpClient,
    ) -> Result<(), Box<dyn Error + Send + Sync>> {
        self.authenticate(credentials, http_client)
            .map_err(Box::new)
            .map_err(Into::into)
    }
}
