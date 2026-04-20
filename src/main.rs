use std::fs;
use std::fs::File;
use std::path::PathBuf;

use clap::{Parser, Subcommand};
use config::Config;
use openhousepopulator::geometry::write_polygons_to_geojson;
use openhousepopulator::{calculate_buildings, AreaId, Error};
use std::io::Write;

/// Populates OSM building data with estimated or given inhabitant counts
#[derive(Parser)]
#[command(author, version, about, long_about = None)]
struct Args {
    #[command(subcommand)]
    command: Option<Commands>,
}

#[derive(Subcommand)]
enum Commands {
    /// calculate and output building population from an OSM PBF file
    Populate {
        /// path to the OSM PBF file
        #[arg(short, long)]
        file_string: String,

        /// inhabitants living in region
        #[arg(short, long)]
        inhabitants: Option<u64>,

        /// if result should be returned using centroids
        #[arg(short, long)]
        centroid: bool,

        /// OSM relation ID to restrict calculation to (e.g. 123456)
        #[arg(long, conflicts_with = "way_id")]
        relation_id: Option<i64>,

        /// OSM way ID to restrict calculation to (e.g. 123456)
        #[arg(long, conflicts_with = "relation_id")]
        way_id: Option<i64>,

        /// output file path (default: ./out/Test.geojson)
        #[arg(short, long)]
        output: Option<PathBuf>,
    },
}

fn main() {
    let cli = Args::parse();

    let settings = Config::builder()
        // Add in `./Settings.toml`
        .add_source(config::File::with_name("settings"))
        .build()
        .expect("Parsing of config file failed.");

    let populator_config = settings
        .try_deserialize::<openhousepopulator::Config>()
        .expect("Parsing of config into crate config failed.");

    match &cli.command {
        Some(Commands::Populate {
            file_string,
            inhabitants,
            centroid,
            relation_id,
            way_id,
            output,
        }) => {
            let area_filter = match (relation_id, way_id) {
                (Some(id), _) => Some(AreaId::Relation(*id)),
                (_, Some(id)) => Some(AreaId::Way(*id)),
                _ => None,
            };

            let file = std::path::Path::new(file_string);
            let r = std::fs::File::open(file).map_err(Error::IOError).unwrap();
            let mut pbf = osmpbfreader::OsmPbfReader::new(r);
            let mut buildings =
                calculate_buildings(&mut pbf, *centroid, &populator_config, area_filter).unwrap();
            match inhabitants {
                Some(inhabitants) => {
                    buildings.distribute_population(*inhabitants, &populator_config)
                }
                None => buildings.estimate_population(),
            }
            println!(
                "Total Population: {}",
                buildings.iter().map(|building| building.pop).sum::<u64>()
            );
            let geojson = write_polygons_to_geojson(&buildings.into_inner());

            let out_path = match output {
                Some(path) => path.clone(),
                None => {
                    let dir = PathBuf::from("./out/");
                    if !dir.exists() {
                        fs::create_dir_all(&dir).unwrap();
                    }
                    dir.join("Test.geojson")
                }
            };
            if let Some(parent) = out_path.parent() {
                if !parent.as_os_str().is_empty() && !parent.exists() {
                    fs::create_dir_all(parent).unwrap();
                }
            }

            let mut file = File::create(&out_path).unwrap();
            write!(file, "{geojson}").unwrap();
        }
        None => {}
    }
}
