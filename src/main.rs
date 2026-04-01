mod weather;

use std::path::Path;
use std::process::ExitCode;

use weather::{OpenMeteoWeatherService, WeatherError, WeatherService};

fn usage(bin_name: &str) -> String {
    format!("Usage: {bin_name} <city|lat,lng>")
}

fn main() -> ExitCode {
    let mut args = std::env::args();
    let bin_name_arg = args.next();
    let bin_name = bin_name_arg
        .as_deref()
        .and_then(|p| Path::new(p).file_name())
        .and_then(|s| s.to_str())
        .unwrap_or("weather");

    let Some(location) = args.next() else {
        eprintln!("{}", usage(bin_name));
        return ExitCode::from(2);
    };

    let service = OpenMeteoWeatherService::new();

    match service.current_temperature_celsius_and_relative_humidity(&location) {
        Ok((temperature_celsius, relative_humidity)) => {
            println!(
                "{}: {:.1}°C (Relative Humidity: {:.0}%)",
                location.trim(),
                temperature_celsius,
                relative_humidity
            );
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
