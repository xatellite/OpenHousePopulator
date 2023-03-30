use geojson::Feature;
use geojson::FeatureCollection;
use geojson::GeoJson;

use crate::pbf::Building;
use crate::pbf::GenericGeometry;

pub fn write_polygons_to_geojson(buildings: &Vec<Building>) -> GeoJson {
    let mut features = vec![];

    for building in buildings {
        let mut tags_map = serde_json::Map::new();
        tags_map.insert("flats".to_string(), building.flats.into());
        tags_map.insert("pop".to_string(), building.pop.into());

        let geometry = match &building.geometry {
            GenericGeometry::GenericPolygon(polygon) => geojson::Geometry::from(polygon),
            GenericGeometry::GenericPoint(point) => geojson::Geometry::from(point),
        };

        let feature = Feature {
            bbox: None,
            geometry: Some(geometry),
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
