pub mod esl_service;
pub mod parse_log;
pub mod poll;
pub mod pricer;
pub mod pricer_service;
use custom_error::custom_error;
use log::debug;
use reqwest::{Certificate, Client, ClientBuilder, Identity, Proxy};
use std::io::Read;
use std::{fs::File, io};

custom_error! {
    /// An error that can occur when building our Api client.
    ///
    /// This error can be seamlessly converted to an `io::Error` and `reqwest::Error` via a `From`
    /// implementation.
    pub ClientError
        Reqwest{source: reqwest::Error} = "Unable to build a reqwest client: {source}",
        Io{source: io::Error}= "unable to read from the file: {source}"
}

/// Reads a certificate file into a Buffer
fn read_certificate(certificate_path: &str) -> Result<Vec<u8>, ClientError> {
    let mut buf = Vec::new();
    File::open(certificate_path)?.read_to_end(&mut buf)?;
    Ok(buf)
}
#[cfg(not(feature = "rustls-tls"))]
fn get_identity(pem_content: Vec<u8>, pkcs8: Vec<u8>) -> Result<Identity, ClientError> {
    Ok(Identity::from_pkcs8_pem(&pem_content, &pkcs8)?)
}
#[cfg(feature = "rustls-tls")]
fn get_identity(pem_content: Vec<u8>, _pkcs8: Vec<u8>) -> Result<Identity, ClientError> {
    Ok(Identity::from_pem(&pem_content)?)
}

/// Builds a configured reqwest http client.
///
/// The client will be configured with both a proxy and a certificate to enable communication
/// and Authentication with our servers.
pub fn build_client(
    proxy_cs: Option<String>,
    certificate_pem: Option<String>,
    certificate_root: Option<String>,
    certificate_key: Option<String>,
) -> Result<Client, ClientError> {
    let mut client_builder = ClientBuilder::new();
    if let Some(cs) = proxy_cs {
        debug!("Config contains a proxy connection string, adding it to the http client");
        let proxy_builder = Proxy::https(cs)?;
        client_builder = client_builder.proxy(proxy_builder)
    }
    if let (Some(pem), Some(root), Some(key)) = (certificate_pem, certificate_root, certificate_key)
    {
        debug!("Config contains a certificate, adding it to the http client");
        let pem_content = read_certificate(&pem)?;
        let pkcs8 = read_certificate(&key)?;
        let root_content = read_certificate(&root)?;

        #[cfg(feature = "rustls-tls")]
        {
            client_builder = client_builder.use_rustls_tls();
        }
        #[cfg(not(feature = "rustls-tls"))]
        {
            client_builder = client_builder.use_native_tls();
        }

        let identity_builder = get_identity(pem_content, pkcs8)?;

        client_builder = client_builder.identity(identity_builder);
        let cert = Certificate::from_pem(&root_content)?;
        client_builder = client_builder.add_root_certificate(cert);
    }
    let client = client_builder.build()?;
    Ok(client)
}
