use faction_xmpp_probe::probe::{self, Config, Report, Status};
use std::{io::Read, time::Duration};
fn main() {
    let report = match arguments() {
        Err(status) => empty(status),
        Ok(cancel) => {
            if std::env::var_os("SSLKEYLOGFILE").is_some()
                || std::env::var_os("TLSKEYLOGFILE").is_some()
            {
                empty(Status::KeylogRefused)
            } else {
                let mut input = String::new();
                match std::io::stdin()
                    .take(8193)
                    .read_to_string(&mut input)
                    .map_err(|_| Status::Config)
                    .and_then(|_| Config::parse(&input))
                {
                    Err(status) => empty(status),
                    Ok(config) => match tokio::runtime::Builder::new_current_thread()
                        .enable_io()
                        .enable_time()
                        .build()
                    {
                        Err(_) => empty(Status::Config),
                        Ok(runtime) => runtime.block_on(probe::run(config, cancel)),
                    },
                }
            }
        }
    };
    println!(
        "{{\"status\":\"{}\",\"attempts\":{},\"received_bytes\":{},\"events\":{}}}",
        report.status.as_str(),
        report.attempts,
        report.received_bytes,
        report.events
    );
    if report.status != Status::Ok {
        std::process::exit(1);
    }
}
fn empty(status: Status) -> Report {
    Report {
        status,
        attempts: 0,
        received_bytes: 0,
        events: 0,
    }
}
fn arguments() -> Result<Option<Duration>, Status> {
    let args: Vec<_> = std::env::args_os()
        .skip(1)
        .take(4)
        .map(|arg| arg.into_string().map_err(|_| Status::Config))
        .collect::<Result<_, _>>()?;
    match args.as_slice() {
        [command] if command == "probe" => Ok(None),
        [command, flag, value] if command == "probe" && flag == "--cancel-after-ms" => {
            let value: u64 = value.parse().map_err(|_| Status::Config)?;
            if value > 10000 {
                return Err(Status::Config);
            }
            Ok(Some(Duration::from_millis(value)))
        }
        _ => Err(Status::Config),
    }
}
