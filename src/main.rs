#[macro_use]
extern crate custom_error;
mod services;
mod settings;
mod ui;
mod utils;
use crossterm::{terminal, ExecutableCommand};
use custom_error::{custom_error, Error};
use log::{debug, error};
use services::{build_client, esl_service::EslServiceError, poll::PollingError, ClientError};
use settings::Settings;
use std::{
    io::{self, stdout},
    time::Duration,
};
use tokio::{task::JoinError, time::sleep};
use tui::{backend::CrosstermBackend, Terminal};

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
        // we already made sure that both useer&password exists
        config.pricer_user.unwrap(),
        config.pricer_password.unwrap(),
        polling_client,
        config.polling_rate,
    )
    .await
    .map_err(|e| e.into())
}

#[tokio::main]
async fn main() -> Result<(), MainError> {
    let drain = tui_logger::Drain::new();
    env_logger::Builder::new()
        .filter_level(log::LevelFilter::Debug)
        .format(move |_buf, record| {
            // patch the env-logger entry through our drain to the tui-logger
            drain.log(record);
            Ok(())
        })
        .init();

    let mut stdout = stdout();
    stdout.execute(terminal::Clear(terminal::ClearType::All))?;

    let app_config = Settings::new()
        .expect("Cannot parse the configuration file, make sure that it is complete");
    debug!("Fetched config from file {:?}", app_config);

    let stdout = io::stdout();
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    let spawn_ui = tokio::task::spawn(async move {
        ui::esl_tui::run_ui(&mut terminal, Duration::from_millis(500))
            .await
            .expect("msg")
    });

    let spawn_poll = tokio::task::spawn(async move {
        { 
            let app_config = app_config.clone();
            app_config.pricer_user.expect("Pricer user is empty in the config file, please add 'pricer_user=<user name>' in hublot-config.toml");
            app_config.pricer_password.expect("Pricer password is empty in the config file, please add 'pricer_password=<password>' in hublot-config.toml");
        }
        loop {
            let app_config = app_config.clone();
            let poller = tokio::task::spawn(async move { polling_worker(app_config).await }).await;
            match poller {
                Ok(output) if output.is_err() => {
                    error!("The poller have crashed from an unrecoverable error. Restarting it in a few seconds");
                    error!("Cause of the crash: {}", output.unwrap_err());
                }
                Ok(_output) => {
                    error!("The poller have stopped with a successfull response. It is not an intended behavior please check the logs above.");
                    break;
                }
                Err(err) if err.is_panic() => {
                    error!("The JoinHandler have crashed from an unrecoverable error. Restarting it in a few seconds");
                }
                Err(_err) => {
                    error!("Runtime shutdown requested, gracefully shutting down the app");
                    // return Err(MainError::JoinError { source: err });
                    break;
                }
            }
            sleep(Duration::from_millis(2000)).await;
        }
    });

    let (poll_result, ui_result) = tokio::join!(spawn_poll, spawn_ui);
    poll_result.expect("Polling stopped for some unknown reason");
    ui_result.expect("Ui stopped for some unknown reason");
    Ok(())
}
