# Weather CLI

Small Rust command-line app that prints the current temperature and relative humidity for a location.

## How It Works

- Accepts one argument: either a city name or coordinates in `lat,lng` format.
- For city input, it geocodes the city with Open-Meteo.
- For coordinate input, it validates latitude/longitude ranges directly.
- Fetches current weather from Open-Meteo and prints a single line result.

## Usage

`location` can be either a city name (for example `Zurich`) or coordinates in `lat,lng` format (for example `47.36667,8.55`).

```bash
cargo run -- "<city|lat,lng>"
```

Examples:

```bash
cargo run -- "Zurich"
cargo run -- "47.36667,8.55"
```

Example output:

```text
Zurich: 17.3°C (Relative Humidity: 61%)
```

## Exit Codes

- `0`: success
- `1`: weather/geocoding request failed or location not found
- `2`: missing argument

## Notes

- Coordinates must be valid ranges:
  - latitude: `-90..90`
  - longitude: `-180..180`
- Network access is required (calls Open-Meteo APIs).
