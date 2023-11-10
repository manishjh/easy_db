use chrono::Local;
use clap::Parser;
use log::{error, info};
use simplelog::{CombinedLogger, Config, TermLogger, WriteLogger};
use std::{
    fmt::Error,
    fs::{create_dir_all, File},
    thread::sleep,
    time::Duration,
};
use utils::consts::{API_GATEWAY_ADDR, API_GATEWAY_ZMQ_PORT};
use zmq::{Context, DEALER, DONTWAIT};

#[derive(Parser, Debug)]
struct Args {
    #[arg(short, long, default_value_t = 1)]
    id: u32,
    #[arg(short, long, default_value_t = 2)]
    count: u32,
}

fn main() -> Result<(), Error> {
    let args = Args::parse();

    setup_logging(&args.id);

    let addr_str = format!("tcp://*:555{}", args.id);

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

    //connect to gateway
    let api_gateway = format!("tcp://{}:{}", API_GATEWAY_ADDR, API_GATEWAY_ZMQ_PORT);
    match socket.connect(&api_gateway) {
        Ok(_) => info!("Succesfully connected to API Gateway"),
        Err(e) => error!("Could not connect to api gateway, err: {:?}", e),
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

        sleep(Duration::from_secs(1));
    }
}

fn setup_logging(id: &u32) {
    let timestamp = Local::now().format("%Y%m%d_%H%M%S").to_string();
    let log_dir = format!("logs/{}", id);
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
