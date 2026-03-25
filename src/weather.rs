use serde::Deserialize;

#[derive(Debug)]
pub enum WeatherError {
    UnknownLocation(String),
    RequestFailed(String),
}

pub trait WeatherService {
    fn current_temp_c(&self, location: &str) -> Result<f64, WeatherError>;
}

pub struct OpenMeteoWeatherService {
    client: reqwest::blocking::Client,
}

#[derive(Debug, PartialEq)]
enum ParsedLocation {
    Coordinates(f64, f64),
    City,
}

impl OpenMeteoWeatherService {
    pub fn new() -> Self {
        Self {
            client: reqwest::blocking::Client::new(),
        }
    }

    fn current_lat_lng(&self, location: &str) -> Result<(f64, f64), WeatherError> {
        let location = location.trim();
        match parse_location_input(location) {
            Ok(ParsedLocation::Coordinates(latitude, longitude)) => return Ok((latitude, longitude)),
            Ok(ParsedLocation::City) => {}
            Err(msg) => {
                return Err(WeatherError::RequestFailed(format!(
                    "invalid coordinates: {msg}"
                )))
            }
        }

        let geo_response = self
            .client
            .get("https://geocoding-api.open-meteo.com/v1/search")
            .query(&[
                ("name", location.to_string()),
                ("count", "1".to_string()),
                ("language", "en".to_string()),
                ("format", "json".to_string()),
            ])
            .send()
            .map_err(|e| WeatherError::RequestFailed(e.to_string()))?
            .error_for_status()
            .map_err(|e| WeatherError::RequestFailed(e.to_string()))?;

        let geo_body: OpenMeteoGeocodingResponse = geo_response
            .json()
            .map_err(|e| WeatherError::RequestFailed(e.to_string()))?;

        geo_body
            .results
            .and_then(|mut r| r.pop())
            .map(|r| (r.latitude, r.longitude))
            .ok_or_else(|| WeatherError::UnknownLocation(location.to_string()))
    }
}

fn parse_location_input(input: &str) -> Result<ParsedLocation, String> {
    if !input.contains(',') {
        return Ok(ParsedLocation::City);
    }

    let mut parts = input.split(',');
    let lat_str = parts
        .next()
        .map(str::trim)
        .filter(|s| !s.is_empty())
        .ok_or_else(|| "expected latitude before comma".to_string())?;
    let lng_str = parts
        .next()
        .map(str::trim)
        .filter(|s| !s.is_empty())
        .ok_or_else(|| "expected longitude after comma".to_string())?;

    if parts.next().is_some() {
        return Err("expected exactly one comma separating lat,lng".to_string());
    }

    let latitude = lat_str
        .parse::<f64>()
        .map_err(|_| "latitude must be a number".to_string())?;
    let longitude = lng_str
        .parse::<f64>()
        .map_err(|_| "longitude must be a number".to_string())?;

    if !(-90.0..=90.0).contains(&latitude) {
        return Err("latitude must be between -90 and 90".to_string());
    }
    if !(-180.0..=180.0).contains(&longitude) {
        return Err("longitude must be between -180 and 180".to_string());
    }

    Ok(ParsedLocation::Coordinates(latitude, longitude))
}

#[derive(Debug, Deserialize)]
struct OpenMeteoResponse {
    current_weather: Option<OpenMeteoCurrentWeather>,
}

#[derive(Debug, Deserialize)]
struct OpenMeteoCurrentWeather {
    temperature: f64,
}

#[derive(Debug, Deserialize)]
struct OpenMeteoGeocodingResponse {
    results: Option<Vec<OpenMeteoGeocodingResult>>,
}

#[derive(Debug, Deserialize)]
struct OpenMeteoGeocodingResult {
    latitude: f64,
    longitude: f64,
}

impl WeatherService for OpenMeteoWeatherService {
    fn current_temp_c(&self, location: &str) -> Result<f64, WeatherError> {
        let (latitude, longitude) = self.current_lat_lng(location)?;

        let resp = self
            .client
            .get("https://api.open-meteo.com/v1/forecast")
            .query(&[
                ("latitude", latitude.to_string()),
                ("longitude", longitude.to_string()),
                ("current_weather", "true".to_string()),
                ("current", "temperature_2m,dew_point_2m,relative_humidity_2m".to_string()),
            ])
            .send()
            .map_err(|e| WeatherError::RequestFailed(e.to_string()))?
            .error_for_status()
            .map_err(|e| WeatherError::RequestFailed(e.to_string()))?;

        let body: OpenMeteoResponse = resp
            .json()
            .map_err(|e| WeatherError::RequestFailed(e.to_string()))?;

        body.current_weather
            .map(|cw| cw.temperature)
            .ok_or_else(|| WeatherError::RequestFailed("missing current_weather".to_string()))
    }
}

#[cfg(test)]
mod tests {
    use super::{parse_location_input, ParsedLocation};

    #[test]
    fn parses_valid_lat_lng() {
        let parsed = parse_location_input("40.7128,-74.0060").expect("should parse");
        assert_eq!(parsed, ParsedLocation::Coordinates(40.7128, -74.006));

        let parsed = parse_location_input(" 51.5074, -0.1278 ").expect("should parse");
        assert_eq!(parsed, ParsedLocation::Coordinates(51.5074, -0.1278));
    }

    #[test]
    fn invalid_format_or_numbers_are_rejected() {
        let parsed = parse_location_input("51.5074 -0.1278").expect("should be city fallback");
        assert_eq!(parsed, ParsedLocation::City);

        let err = parse_location_input("abc,def").expect_err("should fail");
        assert_eq!(err, "latitude must be a number");
    }

    #[test]
    fn out_of_range_coordinates_are_rejected() {
        let err = parse_location_input("95,10").expect_err("should fail");
        assert_eq!(err, "latitude must be between -90 and 90");

        let err = parse_location_input("45,190").expect_err("should fail");
        assert_eq!(err, "longitude must be between -180 and 180");
    }

    #[test]
    fn city_name_falls_back_to_geocoding_path() {
        let parsed = parse_location_input("London").expect("should parse");
        assert_eq!(parsed, ParsedLocation::City);
    }
}
