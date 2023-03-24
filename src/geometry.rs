use std::collections::HashMap;

use geo::Centroid;
use geo::Polygon;
use geojson::Feature;
use geojson::FeatureCollection;
use geojson::GeoJson;
use serde_json::Value;

pub fn write_polygons_to_geojson(
    building_polygons: &Vec<(Polygon, HashMap<String, Value>)>,
    apply_centroid: bool,
) -> GeoJson {
    let mut features = vec![];

    for (geometry, tags) in building_polygons {
        let mut tags_map = serde_json::Map::new();
        for (key, value) in tags {
            tags_map.insert(key.clone(), value.clone());
        }

        let mut geojson_geomentry;

        if apply_centroid {
            let point = geometry.centroid().expect("not a centroid");
            geojson_geomentry = Some(geojson::Geometry::from(&point));
        } else {
            geojson_geomentry = Some(geojson::Geometry::from(geometry));
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
        features: features,
        foreign_members: None,
    })
}
