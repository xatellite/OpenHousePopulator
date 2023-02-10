use pyo3::prelude::*;
use futures::executor::block_on;
mod overpass;
mod populator;

#[derive(Clone)]
#[pyclass]
pub struct Config {
    pub LEVEL_THRESHOLD: usize,
    pub REROLL_THRESHOLD: usize,
    pub  REROLL_PROBABILITY: usize,
    pub  LEVEL_FACTOR: usize,
    pub  HOUSENUMBER_FACTOR: usize,
    pub  REQUEST_URL: String,
}

#[pymethods]
impl Config {
    #[new]
    pub fn new(LEVEL_THRESHOLD: usize, REROLL_THRESHOLD: usize, REROLL_PROBABILITY: usize, LEVEL_FACTOR: usize, HOUSENUMBER_FACTOR: usize, REQUEST_URL: String) -> Config {
        Config {LEVEL_THRESHOLD, REROLL_THRESHOLD, REROLL_PROBABILITY, LEVEL_FACTOR, HOUSENUMBER_FACTOR, REQUEST_URL}
    }
}

/// 
#[pyfunction]
fn populate(district: &str, inhabitants: u64, centroid: bool, config: &Config) -> PyResult<()> {

    let buildings = match block_on(overpass::query_overpass(district, "building", "area", config)) {
        Ok(result) => result,
        Err(error) => panic!("{}", error)
    };

    let house_numbers = match block_on(overpass::query_overpass(district, "addr:housenumber", "node", config)) {
        Ok(result) => result,
        Err(error) => panic!("{}", error)
    };

    populator::count_inhabitants(buildings, house_numbers, inhabitants, district, centroid, config);

    Ok(())
}

/// A Python module implemented in Rust.
#[pymodule]
fn openhousepopulator_core(_py: Python, m: &PyModule) -> PyResult<()> {
    m.add_class::<Config>()?;
    m.add_function(wrap_pyfunction!(populate, m)?)?;
    Ok(())
}