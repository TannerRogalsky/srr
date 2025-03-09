use futures::{FutureExt, TryFutureExt};
pub use reqwest::Error as ReqwestError;

#[derive(Debug)]
pub enum Error {
    NotFound,
    Request(ReqwestError),
}

impl From<ReqwestError> for Error {
    fn from(value: ReqwestError) -> Self {
        Self::Request(value)
    }
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Error::NotFound => f.write_str("Error(NotFound)"),
            Error::Request(error) => error.fmt(f),
        }
    }
}

impl std::error::Error for Error {}

pub struct Client {
    inner: reqwest::Client,
    details_url: reqwest::Url,
    download_url: reqwest::Url,
}

impl Default for Client {
    fn default() -> Self {
        Self::new()
    }
}

impl Client {
    pub fn new() -> Self {
        let inner = reqwest::Client::new();
        let details_url = reqwest::Url::parse("https://api.srrdb.com/v1/details/").unwrap();
        let download_url = reqwest::Url::parse("https://www.srrdb.com/download/file/").unwrap();
        Self {
            inner,
            details_url,
            download_url,
        }
    }

    pub fn details_request<'a, R: Into<DetailsRequest<'a>>>(
        &self,
        request: R,
    ) -> impl futures::Future<Output = Result<DetailsResponse, Error>> {
        let request = request.into();
        let url = self.details_url.join(&request.release_name).unwrap();
        let request = self.inner.get(url).build().unwrap();
        self.inner
            .execute(request)
            .and_then(|response| response.json::<DetailsOrNotFound>())
            .err_into::<Error>()
            .and_then(|response| {
                futures::future::ready(match response {
                    DetailsOrNotFound::NotFound(_) => Err(Error::NotFound),
                    DetailsOrNotFound::Details(details_response) => Ok(details_response),
                })
            })
    }

    pub fn file_request(
        &self,
        request: FileRequest,
    ) -> impl futures::Future<Output = Result<Vec<u8>, Error>> {
        let input = format!("{}/{}", request.base, request.details.name);
        let url = self.download_url.join(&input).unwrap();
        let request = self.inner.get(url).build().unwrap();
        self.inner
            .execute(request)
            .and_then(|response| response.bytes())
            .map_ok(Into::into)
            .err_into()
    }
}

pub struct DetailsRequest<'a> {
    pub release_name: std::borrow::Cow<'a, str>,
}

impl<'a> From<&'a str> for DetailsRequest<'a> {
    fn from(release_name: &'a str) -> Self {
        Self {
            release_name: std::borrow::Cow::Borrowed(release_name),
        }
    }
}

#[derive(Debug, serde::Deserialize)]
#[serde(untagged)]
enum DetailsOrNotFound {
    NotFound([(); 0]),
    Details(DetailsResponse),
}

#[derive(Debug, serde::Deserialize)]
pub struct DetailsResponse {
    pub name: String,
    pub files: Vec<FileDetails>,
    #[serde(rename = "archived-files")]
    pub archived_files: Vec<FileDetails>,
}

impl DetailsResponse {
    pub fn file_request(&self, name: &str) -> Option<FileRequest> {
        self.files.iter().find_map(|details| {
            (details.name == name).then_some(FileRequest {
                base: &self.name,
                details,
            })
        })
    }
}

pub struct FileRequest<'a> {
    base: &'a str,
    details: &'a FileDetails,
}

#[derive(Debug, serde::Deserialize)]
pub struct FileDetails {
    pub name: String,
    pub size: u32,
    // I think this is the u16 hex encoded
    pub crc: String,
}

impl<'a> tower_service::Service<DetailsRequest<'a>> for Client {
    type Response = DetailsResponse;
    type Error = Error;
    type Future = futures::future::BoxFuture<'a, Result<Self::Response, Self::Error>>;

    fn poll_ready(
        &mut self,
        _cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<Result<(), Self::Error>> {
        std::task::Poll::Ready(Ok(()))
    }

    fn call(&mut self, req: DetailsRequest<'a>) -> Self::Future {
        self.details_request(req).boxed()
    }
}

impl<'a> tower_service::Service<FileRequest<'a>> for Client {
    type Response = Vec<u8>;
    type Error = Error;
    type Future = futures::future::BoxFuture<'a, Result<Self::Response, Self::Error>>;

    fn poll_ready(
        &mut self,
        _cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<Result<(), Self::Error>> {
        std::task::Poll::Ready(Ok(()))
    }

    fn call(&mut self, req: FileRequest<'a>) -> Self::Future {
        self.file_request(req).boxed()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn it_works() {
        let client = Client::new();
        let response = client
            .details_request(
                "Harry.Potter.And.The.Chamber.Of.Secrets.2002.DVDRip.XViD-iNTERNAL-TDF",
            )
            .await
            .unwrap();
        let response = client
            .file_request(response.file_request("tdf-hpatcos.nfo").unwrap())
            .await
            .unwrap();
        let s = String::from_utf8_lossy(&response);
        assert!(s.contains("Harry.Potter.And.The.Chamber.Of.Secrets.2002.DVDRip.XViD-iNTERNAL-TDF"));
    }
}
