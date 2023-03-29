#![feature(async_fn_in_trait)]
#[macro_use]
extern crate custom_error;
mod services;
mod settings;
mod utils;
use chrono::Local;
use console::{style, Emoji, Term};
use custom_error::{custom_error, Error};
use env_logger::Env;
use esl_utils::parse::ParseClient;

use log::{debug, error};
use reqwest::StatusCode;
use services::pricer_service::PricerError;
use services::{build_client, esl_service::EslServiceError, poll::PollingError, ClientError};
use settings::Settings;

use std::io::Write;
use std::{
    fs::File,
    io::{self},
    time::Duration,
};
use tokio::{task::JoinError, time::sleep};

use crate::services::parse_log::ParseLog;

#[cfg(target_family = "windows")]
static LOOKING_GLASS: Emoji<'_, '_> = Emoji("🔍  ", "Θ  ");
#[cfg(target_family = "unix")]
static LOOKING_GLASS: Emoji<'_, '_> = Emoji("🔍  ", "🔍  ");
#[cfg(target_family = "windows")]
static CONFIG: Emoji<'_, '_> = Emoji("⚙️  ", "⌂  ");
#[cfg(target_family = "unix")]
static CONFIG: Emoji<'_, '_> = Emoji("⚙️  ", "⚙️  ");

#[cfg(target_family = "windows")]
static ROCKET: Emoji<'_, '_> = Emoji("🚀  ", "☼  ");
#[cfg(target_family = "unix")]
static ROCKET: Emoji<'_, '_> = Emoji("🚀  ", "🚀  ");

#[cfg(target_family = "windows")]
macro_rules! LOG_PLACEHOLDER {
    () => {
        "{}:{} {} [{}] - {} \r\n"
    };
}

#[cfg(target_family = "unix")]
macro_rules! LOG_PLACEHOLDER {
    () => {
        "{}:{} {} [{}] - {}"
    };
}

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
        PricerError{source: PricerError} = "An issue occured while calling the Pricer Server: {source}",
        Todo = "TODO: Missing implementation"
}

/// the background_task that starts the polling worker and updates the display of the ESLs
async fn polling_worker(config: Settings) -> Result<(), MainError> {
    let polling_client = build_client(
        config.proxy_cs,
        config.certificate_pem_path,
        config.certificate_root_path,
    )?;

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
    let t = Term::stdout();
    t.clear_screen()?;
    let log_file = Box::new(File::create("hublot-logs.txt").expect("Can't create log file"));

    let app_config = Settings::new()
        .expect("Cannot parse the configuration file, make sure that it is complete");

    let log_level = app_config.clone().log_level.unwrap_or("warn".to_string());
    let envconf = Env::default().default_filter_or(&log_level);
    let parse_client = ParseClient::new(
        app_config
            .clone()
            .parse_id
            .expect("Missing parse configuration key: [parse_id]"),
        None,
        app_config
            .clone()
            .parse_url
            .expect("Missing parse configuration key: [parse_url]"),
    );
    let log_config = app_config.clone();




    env_logger::Builder::from_env(envconf)
        .format(move |buf, record| {
            writeln!(
                buf,
                LOG_PLACEHOLDER!(),
                record.file_static().unwrap_or("unknown"),
                record.line().unwrap_or(0),
                Local::now().format("%Y-%m-%dT%H:%M:%S%.3f"),
                record.level(),
                record.args()
            )
            .expect("Cannot write log to file");
            if record.target().contains("esl_services_backend") {
                let parse_client = parse_client.clone();
                let log_config = log_config.clone();
                let log = ParseLog {
                    app: "hublot-esl-backend".to_string(),
                    level: record.level().to_string(),
                    message: record.args().to_string(),
                    serial: log_config.client_serial,
                };
                tokio::task::spawn(async move {
                    parse_client
                        .clone()
                        .save("parse/classes/Log".to_string(), log)
                        .await
                });
            }
            Ok(())
        })
        .target(env_logger::Target::Pipe(log_file))
        .init();

    let logo = include_str!("../logo.ansi.txt");

    println!("{logo}");

    println!(
        "{} {}Loading app configuration...",
        style("[1/4]").bold().dim(),
        CONFIG
    );


    debug!("Fetched config from file {:?}", app_config);

    // test proxy and log 
    {
        let app_config = app_config.clone();
        let client = build_client(
            app_config.proxy_cs,
            app_config.certificate_pem_path,
            app_config.certificate_root_path,
        )?;
        let res = client.get(format!("{}/esl-api/status", app_config.hublot_server_url)).send().await.expect(
            "Test connection has failed, please make sure that the proxy configuration is correct"
        );
        match res.status() {
            StatusCode::OK => println!(
                "{} {} Connection is OK, proxy and certificate are valids",
                style("[2/4]").bold().dim(),
                CONFIG
            ),
            _ => println!(
                "{} {} Connection to our server failed, please make sure that the proxy configuration is correct",
                style("[2/4]").bold().dim(),
                CONFIG
            )
        }
    }

    let spawn_poll = tokio::task::spawn(async move {
        {
            println!(
                "{} {}Checking if the config is complete...",
                style("[3/4]").bold().dim(),
                LOOKING_GLASS
            );
            let app_config = app_config.clone();
            app_config.pricer_user.expect("Pricer user is empty in the config file, please add 'pricer_user=<user name>' in hublot-config.toml");
            app_config.pricer_password.expect("Pricer password is empty in the config file, please add 'pricer_password=<password>' in hublot-config.toml");
            println!(
                "{} {}Starting the application loop...",
                style("[4/4]").bold().dim(),
                ROCKET
            );
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
    let (poll_result,) = tokio::join!(spawn_poll);
    poll_result.expect("Polling stopped for some unknown reason");
    Ok(())
}
