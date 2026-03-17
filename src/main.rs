mod weather;

use std::path::Path;
use std::process::ExitCode;

use weather::{OpenMeteoWeatherService, WeatherError, WeatherService};

fn usage(bin_name: &str) -> String {
    format!("Usage: {bin_name} <location>")
}

fn main() -> ExitCode {
    let mut args = std::env::args();
    let bin_name = args
        .next()
        .as_deref()
        .and_then(|p| Path::new(p).file_name())
        .and_then(|s| s.to_str())
        .unwrap_or("weather")
        .to_string();

    let Some(location) = args.next() else {
        eprintln!("{}", usage(&bin_name));
        return ExitCode::from(2);
    };

    let service = OpenMeteoWeatherService::new();

    match service.current_temp_c(&location) {
        Ok(temp_c) => {
            println!("{}: {:.1}°C", location.trim(), temp_c);
            ExitCode::SUCCESS
        }
        Err(WeatherError::UnknownLocation(name)) => {
            eprintln!("Unknown location: {name}");
            ExitCode::from(1)
        }
        Err(WeatherError::RequestFailed(msg)) => {
            eprintln!("Weather request failed: {msg}");
            ExitCode::from(1)
        }
    }
}
