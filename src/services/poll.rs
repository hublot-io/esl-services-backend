use std::{io, time::Duration};
use super::{esl_service::EslServiceError, pricer_service::PricerError};
use crate::services::{
    esl_service::get_print_requests,
    pricer_service::{self, PricerEsl},
};
use log::debug;
use reqwest::Client;
use tokio::time::sleep;

custom_error! {
    /// An error that can occur when during the API.
    ///
    /// This error can be seamlessly converted to an `io::Error` and `reqwest::Error` via a `From`
    /// implementation.
    pub PollingError
        EslServiceError{source: EslServiceError} = "An issue occured calling the EslService: {source}",
        Reqwest{source: reqwest::Error} = "An issue occured within this request: {source}",
        PricerError{source: PricerError} = "An issue occured calling the PricerService: {source}",
        Io{source: io::Error}= "An I/O error occured: {source}",
}

/// A polling workser that fetches the `server_url` every `poll_interval`.
///
/// If the server response is not empty, the worker will send a request to the `esl_server` in order
/// to update the display of the ESLs
pub async fn poll(
    client_serial: &str,
    hublot_server_url: &str,
    esl_server_url: &str,
    pricer_user: Option<String>,
    pricer_password: Option<String>,
    client: Client,
    polling_rate: Option<i32>,
) -> Result<(), PollingError> {
    let interval = polling_rate.unwrap_or(2000);
    debug!("Starting polling loop");
    loop {
        let print_requests = get_print_requests(hublot_server_url, &client, client_serial).await?;

        let pricer_requests: Vec<PricerEsl> = print_requests
            .iter()
            .map(|esl| pricer_service::from_esl(esl.esl.esl.clone()))
            .collect();

        for pricer_esl in pricer_requests {
            pricer_service::update_esl(
                pricer_esl,
                esl_server_url,
                pricer_user.clone(),
                pricer_password.clone(),
            )
            .await?;
        }
        // wait for a few seconds before starting a new poll
        sleep(Duration::from_millis(interval as u64)).await;
    }
}
