use std::collections::HashMap;

#[derive(Debug)]
pub enum WeatherError {
    UnknownLocation(String),
}

pub trait WeatherService {
    fn current_temp_c(&self, location: &str) -> Result<f64, WeatherError>;
}

pub struct MockWeatherService {
    temps_c_by_location: HashMap<String, f64>,
}

impl MockWeatherService {
    pub fn new() -> Self {
        let temps_c_by_location: HashMap<String, f64> = [
            ("berlin".to_string(), 12.3),
            ("london".to_string(), 10.1),
            ("new york".to_string(), 6.4),
            ("tokyo".to_string(), 15.8),
        ]
        .into_iter()
        .collect();

        Self { temps_c_by_location }
    }
}

impl WeatherService for MockWeatherService {
    fn current_temp_c(&self, location: &str) -> Result<f64, WeatherError> {
        let key = location.trim().to_ascii_lowercase();
        self.temps_c_by_location
            .get(&key)
            .copied()
            .ok_or_else(|| WeatherError::UnknownLocation(location.trim().to_string()))
    }
}

