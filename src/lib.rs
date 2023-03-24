//! # OpenHousePopulator
//!
//! This tool automatically distributes a given amount of inhabitants to osm buildings.
//! The calculation is based on predefined heuristics, calculating a flat count per building and randomly distributing people.

mod geometry;
mod datalayer;
mod parser;

use std::fmt::Display;
use std::path::Path;
use datalayer::{is_building, is_housenumber_node, load_buildings};
use geo::Polygon;
use crate::geometry::{write_polygons_to_geojson};

#[derive(Debug)]
pub enum Error {
    OverpassError(reqwest::Error),
    IOError(std::io::Error),
}

impl Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::OverpassError(err) => write!(f, "failed to query overpass api: {}", err),
            Self::IOError(err) => write!(f, "io error occured: {}", err),
            _ => write!(f, "some error occured"),
        }
    }
}

impl std::error::Error for Error {}

#[derive(Clone, serde::Deserialize)]
pub struct Config {
    pub level_threshold: usize,
    pub reroll_threshold: usize,
    pub reroll_probability: usize,
    pub level_factor: usize,
    pub housenumber_factor: usize,
    pub request_url: String,
}

/// Takes pbf and inhabitants count and calculates geojson
pub fn spread_population(
    file: &Path,
    inhabitants: &u64,
    centroid: &bool,
    config: &Config,
) -> Result<(), Error> {

  // Read pbf file

    let r = std::fs::File::open(file).map_err(|err| Error::IOError((err)))?; // ToDo Handle error
    let mut pbf = osmpbfreader::OsmPbfReader::new(r);

    let osm_buildings = pbf.get_objs_and_deps(is_building).unwrap();
    let osm_housenumbers = pbf.get_objs_and_deps(is_housenumber_node).unwrap();
    let buildings = load_buildings(osm_buildings, osm_housenumbers);
    // println!("{:?}", buildings);
    Ok(())
}
