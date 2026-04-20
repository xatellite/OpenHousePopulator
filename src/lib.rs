//! # OpenHousePopulator
//!
//! This tool automatically distributes a given amount of inhabitants to osm buildings.
//! The calculation is based on predefined heuristics, calculating a flat count per building and randomly distributing people.

mod config;
pub mod geometry;
mod parser;
mod pbf;

use indicatif::{ProgressBar, ProgressStyle};
use osmpbfreader::OsmPbfReader;
use pbf::{
    is_area_relation, is_area_way, is_building, is_exclude_area, is_housenumber_node,
    load_area_relation_polygon, load_area_way_polygon, load_housenumbers, load_ways,
};

use std::fmt::Display;

pub use crate::config::Config;
pub use crate::pbf::{AreaId, Building, Buildings, GenericGeometry};

#[derive(Debug)]
pub enum Error {
    OverpassError(reqwest::Error),
    IOError(std::io::Error),
}

impl Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::OverpassError(err) => write!(f, "failed to query overpass api: {err}"),
            Self::IOError(err) => write!(f, "io error occured: {err}"),
        }
    }
}

impl std::error::Error for Error {}

/// Calculates the population of houses in a given pbf
pub fn calculate_buildings<T: std::io::Read + std::io::Seek>(
    pbf: &mut OsmPbfReader<T>,
    centroid: bool,
    config: &Config,
    area_filter: Option<AreaId>,
) -> Result<Buildings, Error> {
    let spinner_style = ProgressStyle::with_template("{spinner:.green} {msg}").unwrap();

    // Load area boundary polygon if requested
    let area_polygon = match &area_filter {
        Some(AreaId::Way(id)) => {
            let pb = ProgressBar::new_spinner();
            pb.set_style(spinner_style.clone());
            pb.set_message(format!("Loading area boundary from way {}...", id));
            pb.enable_steady_tick(std::time::Duration::from_millis(100));
            let objs = pbf.get_objs_and_deps(|obj| is_area_way(obj, *id)).unwrap();
            let poly = load_area_way_polygon(&objs, *id);
            pb.finish_with_message(format!(
                "Area boundary from way {} {}",
                id,
                if poly.is_some() { "loaded" } else { "not found" }
            ));
            if poly.is_none() {
                log::warn!("Way {} not found or has insufficient nodes.", id);
            }
            poly
        }
        Some(AreaId::Relation(id)) => {
            let pb = ProgressBar::new_spinner();
            pb.set_style(spinner_style.clone());
            pb.set_message(format!("Loading area boundary from relation {}...", id));
            pb.enable_steady_tick(std::time::Duration::from_millis(100));
            let objs = pbf
                .get_objs_and_deps(|obj| is_area_relation(obj, *id))
                .unwrap();
            let poly = load_area_relation_polygon(&objs, *id);
            pb.finish_with_message(format!(
                "Area boundary from relation {} {}",
                id,
                if poly.is_some() { "loaded" } else { "not found" }
            ));
            if poly.is_none() {
                log::warn!("Relation {} not found or has no outer members.", id);
            }
            poly
        }
        None => None,
    };

    // Retrieve objects from pbf
    let pb = ProgressBar::new_spinner();
    pb.set_style(spinner_style.clone());
    pb.set_message("Loading buildings from pbf...");
    pb.enable_steady_tick(std::time::Duration::from_millis(100));
    let osm_buildings = pbf.get_objs_and_deps(is_building).unwrap();
    pb.finish_with_message(format!("Loaded {} building objects", osm_buildings.len()));

    let pb = ProgressBar::new_spinner();
    pb.set_style(spinner_style.clone());
    pb.set_message("Loading housenumber nodes from pbf...");
    pb.enable_steady_tick(std::time::Duration::from_millis(100));
    let osm_housenumbers = pbf.get_objs_and_deps(is_housenumber_node).unwrap();
    pb.finish_with_message(format!(
        "Loaded {} housenumber nodes",
        osm_housenumbers.len()
    ));

    let pb = ProgressBar::new_spinner();
    pb.set_style(spinner_style.clone());
    pb.set_message("Loading exclude areas from pbf...");
    pb.enable_steady_tick(std::time::Duration::from_millis(100));
    let osm_exclude_areas = pbf
        .get_objs_and_deps(|obj| is_exclude_area(obj, config))
        .unwrap();
    pb.finish_with_message(format!(
        "Loaded {} exclude area objects",
        osm_exclude_areas.len()
    ));

    let building_ways = load_ways(osm_buildings);
    let housenumbers = load_housenumbers(osm_housenumbers);
    let mut buildings = Buildings::from((building_ways, &housenumbers, config));

    if let Some(polygon) = area_polygon {
        let pb = ProgressBar::new_spinner();
        pb.set_style(spinner_style.clone());
        pb.set_message("Filtering buildings within area boundary...");
        pb.enable_steady_tick(std::time::Duration::from_millis(100));
        let before = buildings.iter().count();
        buildings = buildings.filter_within(&polygon);
        pb.finish_with_message(format!(
            "Filtered to {} buildings (from {})",
            buildings.iter().count(),
            before
        ));
    }

    let areas = load_ways(osm_exclude_areas);
    if centroid {
        buildings.centroid();
    }
    buildings = buildings.exclude_in(&areas);

    Ok(buildings)
}
