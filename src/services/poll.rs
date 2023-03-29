use super::{esl_service::EslServiceError, pricer_service::PricerError};
use crate::services::{
    esl_service::get_print_requests,
    pricer_service::{self, PricerEsl},
};
use indicatif::{MultiProgress, ProgressBar, ProgressStyle};
use log::info;
use reqwest::Client;
use std::{io, time::Duration};
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
    pricer_user: String,
    pricer_password: String,
    client: Client,
    polling_rate: Option<i32>,
) -> Result<(), PollingError> {
    let interval = polling_rate.unwrap_or(2000);
    let spinner_style = ProgressStyle
        ::with_template("{prefix:.bold.dim} {spinner} {wide_msg}")
        .unwrap()
        .tick_chars("/|\\- ");

    let m = MultiProgress::new();
    let pb = m.add(ProgressBar::new(5));
    pb.set_style(spinner_style.clone());
    pb.set_prefix(format!("[{}/∞]", 0));

    loop {
        pb.set_message("polling_broker: Getting print requests".to_string());
        let print_requests = get_print_requests(hublot_server_url, &client, client_serial).await?;
        pb.inc(1);
        let pricer_requests: Vec<PricerEsl> = print_requests
            .iter()
            .map(|esl| esl.clone().into())
            .collect();

        if !pricer_requests.is_empty() {
            let ids: Vec<String> = pricer_requests
                .iter()
                .map(|p| p.barcode.to_string())
                .collect();
            info!(
                "Found {} esls to update, ids: [{:?}]",
                pricer_requests.len(),
                ids
            );
            pb.set_message(format!("{} print request found", pricer_requests.len()));
        }

        for (i, pricer_esl) in pricer_requests.iter().enumerate() {
            pb.set_message(format!(
                "{}/{} Attempting to update an ESL",
                i + 1,
                pricer_requests.len()
            ));
            pricer_service::on_poll(
                pricer_esl.clone(),
                esl_server_url,
                pricer_user.clone(),
                pricer_password.clone(),
                &pb,
            )
            .await?;
        }
        // Divide the time we have to wait so we can animate the spinner
        let mut wait = 0;
        let time = 150;
        pb.set_message("Waiting for a new update".to_string());
        loop {
            pb.inc(1);
            sleep(Duration::from_millis(time as u64)).await;
            wait += time;
            if wait >= interval {
                break;
            }
        }
    }
}
