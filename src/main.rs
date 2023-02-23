#[macro_use]
extern crate custom_error;
mod services;
mod settings;
mod utils;
use std::{io, time::Duration};

use config;
use custom_error::{custom_error, Error};
use log::{debug, error, info};
use services::{build_client, esl_service::EslServiceError, poll::PollingError, ClientError};
use settings::Settings;
use tokio::{task::JoinError, time::sleep};

custom_error! {
    /// An error that can occur when the lifetime of the App.
    ///
    /// This error can be seamlessly converted to an `io::Error` and `reqwest::Error` via a `From`
    /// implementation.
    pub MainError
        EslServiceError{source: EslServiceError} = "An issue occured calling the EslService: {source}",
        ClientError{source: ClientError} = "An issue occured with the ClientBuilder",
        Reqwest{source: reqwest::Error} = "An issue occured within this request: {source}",
        Io{source: io::Error}= "An I/O error occured: {source}",
        PollingError{source: PollingError}= @{ format!("An error occured with the polling service: {:?}", source.source().unwrap() )} ,
        JoinError{source: JoinError}= "A tokio error occured while joining our process loop : {source}",

}

/// the background_task that starts the polling worker and updates the display of the ESLs
async fn polling_worker(config: Settings) -> Result<(), MainError> {
    let polling_client = build_client(config.proxy_cs, config.certificate_path)?;
    services::poll::poll(
        &config.client_serial,
        &config.hublot_server_url,
        &config.esl_server_url,
        config.pricer_user,
        config.pricer_password,
        polling_client,
        config.polling_rate,
    )
    .await
    .map_err(|e| e.into())
}

#[tokio::main]
async fn main() -> Result<(), MainError> {
    env_logger::init();
    let app_config = Settings::new()
        .expect("Cannot parse the configuration file, make sure that it is well written");
    debug!("Fetched config from file {:?}", app_config);

    loop {
        let app_config = app_config.clone();
        let poller = tokio::task::spawn(async move { polling_worker(app_config).await }).await;
        match poller {
            Ok(output) if output.is_err() => {
                error!("The poller have crashed from an unrecoverable error. Restarting it in a few seconds");
                error!("Cause of the crash: {}", output.unwrap_err());
            },
            Ok(output) => {
                error!("The poller have stopped with a successfull response. It is not an intended behavior please check the logs above.");
                break;
            }
            Err(err) if err.is_panic() => {
                error!("The JoinHandler have crashed from an unrecoverable error. Restarting it in a few seconds");
            }
            Err(err) => {
                error!("Runtime shutdown requested, gracefully shutting down the app");
                return Err(MainError::JoinError { source: err });
            }
        }
        sleep(Duration::from_millis(2000)).await;
    }
    return Ok(());
}
