use clap::Parser;
use std::{
    fs::{create_dir_all, File},
    thread, time,
};
//use actix_web::{error::InternalError, http::StatusCode};
#[allow(dead_code)]
use actix_web::{web, App, HttpServer};
use chrono::Local;
use log::{error, info};
use serde::Deserialize;
use simplelog::{CombinedLogger, Config, TermLogger, WriteLogger};
use utils::consts::{API_GATEWAY_HTTP_PORT, API_GATEWAY_ZMQ_PORT};
use zmq::{Context, SocketType::DEALER, DONTWAIT};

mod http_handlers;

#[derive(Deserialize)]
pub struct KeyValue {
    key: String,
    value: String,
}

#[derive(Parser, Debug)]
struct Args {
    #[arg(short, long, default_value_t = 1)]
    id: u32,
    #[arg(short, long, default_value_t = 2)]
    count: u32,
}

#[actix_web::main]
//async fn main() -> std::io::Result<()> {
async fn main() -> Result<(), anyhow::Error> {
    let args = Args::parse();
    //Initialize Logging
    setup_logging();

    let zmq_thread = thread::spawn(move || {
        info!("starting zmq dealer socket on gateway");

        let addr_str = format!("tcp://*:{}", API_GATEWAY_ZMQ_PORT);

        info!("going to start server on {}", &addr_str);

        let context = Context::new();
        let socket = match context.socket(DEALER) {
            Ok(socket) => socket,
            Err(e) => {
                error!("Could not create dealer socket, err: {:?}", e);
                panic!("exiting due to error");
            }
        };

        //server setup bind to addr.
        match socket.bind(&addr_str) {
            Ok(_) => info!("server started on : {}", &addr_str),
            Err(e) => error!("Could not bind to socket addr: {}, err: {:?}", &addr_str, e),
        };

        //client setup, connect to all raft sockets.
        for i in 1..=args.count {
            if i == args.id {
                continue;
            }
            let target_addr = format!("tcp://localhost:555{}", i);
            match socket.connect(&target_addr) {
                Ok(_) => info!("connected to : {}", &target_addr),
                Err(e) => error!(
                    "Could not connect to socket addr: {}, err: {:?}",
                    &target_addr, e
                ),
            }
        }

        loop {
            match socket.recv_string(DONTWAIT) {
                Ok(result) => match result {
                    Ok(message) => {
                        info!("received : {}", message)
                        // parse message, process accordingly.
                        //msg format:
                        //{
                        //type: {}
                        //data: {}
                        //from: {}
                        //to : {}
                        //}
                    }
                    Err(e) => error!("Failed to receive {:?}", e),
                },
                Err(e) => error!("Failed to receive {:?}", e),
            }

            match socket.send(format!("request from {}", &args.id).as_str(), 0) {
                Ok(_) => {}
                Err(e) => error!("Could not send request {:?}", e),
            }

            thread::sleep(time::Duration::from_secs(1));
        }
    });

    info!("Starting http api gateway");

    //start the http server
    HttpServer::new(move || {
        App::new()
            .route("/kv/{key}", web::get().to(http_handlers::get))
            .route("/kv", web::post().to(http_handlers::post))
    })
    .bind(format!("0.0.0.0:{}", API_GATEWAY_HTTP_PORT))?
    .run()
    .await?;

    match zmq_thread.join() {
        Ok(_) => {}
        Err(e) => error!("ZMQ thread has panicked!, {:?}", e),
    }

    Ok(())
}

fn setup_logging() {
    let timestamp = Local::now().format("%Y%m%d_%H%M%S").to_string();
    let log_dir = format!("logs/{}", "gateway");
    let log_file = format!("{}/app_{}.log", log_dir, timestamp);

    create_dir_all(&log_dir).unwrap();

    CombinedLogger::init(vec![
        TermLogger::new(
            log::LevelFilter::Debug,
            Config::default(),
            simplelog::TerminalMode::Mixed,
            simplelog::ColorChoice::Auto,
        ),
        WriteLogger::new(
            log::LevelFilter::Debug,
            Config::default(),
            File::create(log_file).unwrap(),
        ),
    ])
    .expect("error setting up logger");
}

// pub trait IntoCustomError<T> {
//     fn http_error(
//         self,
//         message: &str,
//         status_code: StatusCode,
//     ) -> core::result::Result<T, actix_web::Error>;

//     fn internal_error(self, message: &str) -> core::result::Result<T, actix_web::Error>
//     where
//         Self: std::marker::Sized,
//     {
//         self.http_error(message, StatusCode::INTERNAL_SERVER_ERROR)
//     }
// }

// impl<T, E: std::fmt::Debug> IntoCustomError<T> for core::result::Result<T, E> {
//     fn http_error(
//         self,
//         message: &str,
//         status_code: StatusCode,
//     ) -> core::result::Result<T, actix_web::Error> {
//         match self {
//             Ok(val) => Ok(val),
//             Err(err) => {
//                 log::error!("error: {:?}", err);
//                 Err(InternalError::new(message.to_string(), status_code).into())
//             }
//         }
//     }
// }
