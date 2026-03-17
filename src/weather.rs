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

impl OpenMeteoWeatherService {
    pub fn new() -> Self {
        Self {
            client: reqwest::blocking::Client::new(),
        }
    }

    fn current_lat_lng(&self, location: &str) -> Result<(f64, f64), WeatherError> {
        let location = location.trim();

        let geo_resp = self
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

        let geo_body: OpenMeteoGeocodingResponse = geo_resp
            .json()
            .map_err(|e| WeatherError::RequestFailed(e.to_string()))?;

        geo_body
            .results
            .and_then(|mut r| r.pop())
            .map(|r| (r.latitude, r.longitude))
            .ok_or_else(|| WeatherError::UnknownLocation(location.to_string()))
    }
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
