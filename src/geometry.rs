use std::collections::HashMap;

use geo::Centroid;
use geo::Polygon;
use geo::polygon;
use geojson::Feature;
use geojson::FeatureCollection;
use geojson::GeoJson;
use osmpbfreader::Tags;
use serde_json::Value;

use crate::datalayer::Building;

pub fn write_polygons_to_geojson(
    buildings: &Vec<Building>,
    apply_centroid: bool,
) -> GeoJson {
    let mut features = vec![];

    for building in buildings {
        let mut tags_map = serde_json::Map::new();
        building.tags.iter().for_each(|(k, v)| {
            tags_map.insert(k.to_string(), Value::String(v.to_string()));
        });

        let geojson_geomentry;

        if apply_centroid {
            let point = building.polygon.centroid().expect("not a centroid");
            geojson_geomentry = Some(geojson::Geometry::from(&point));
        } else {
            geojson_geomentry = Some(geojson::Geometry::from(&building.polygon));
        }

        let feature = Feature {
            bbox: None,
            geometry: geojson_geomentry,
            id: None,
            properties: Some(tags_map),
            foreign_members: None,
        };
        features.push(feature);
    }

    GeoJson::from(FeatureCollection {
        bbox: None,
        features,
        foreign_members: None,
    })
}
