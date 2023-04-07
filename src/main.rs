use std::fs::File;
use std::path::PathBuf;

use clap::{Parser, Subcommand};
use config::Config;
use openhousepopulator::geometry::write_polygons_to_geojson;
use openhousepopulator::{calculate_buildings, Error};
use std::io::Write;

/// Simple program to greet a person
#[derive(Parser)]
#[command(author, version, about, long_about = None)]
struct Args {
    #[command(subcommand)]
    command: Option<Commands>,
}

#[derive(Subcommand)]
enum Commands {
    /// populates area by region name
    Populate {
        /// name of the region
        #[arg(short, long)]
        file_string: String,

        /// inhabitants living in region
        #[arg(short, long)]
        inhabitants: Option<u64>,

        /// if result should be returned using centroids
        #[arg(short, long)]
        centroid: bool,
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
        }) => {
            let file = std::path::Path::new(file_string);
            let r = std::fs::File::open(file).map_err(Error::IOError).unwrap();
            let mut pbf = osmpbfreader::OsmPbfReader::new(r);
            let mut buildings =
                calculate_buildings(&mut pbf, *centroid, &populator_config).unwrap();
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

            // Create a temporary file.
            let temp_directory = PathBuf::from("./out/");
            let file_name = "Test.geojson";
            let temp_file = temp_directory.join(file_name);

            let mut file = File::create(temp_file).unwrap();
            write!(file, "{geojson}").unwrap();
        }
        None => {}
    }
}
