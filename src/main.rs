use clap::{Parser, Subcommand};
use config::Config;
use openhousepopulator::{spread_population, get_area_by_point};

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
        name: String,

        /// inhabitants living in region
        #[arg(short, long)]
        population: u64,

        /// if result should be returned using centroids
        #[arg(short, long)]
        centroid: bool,
    },

    /// gives name and area of region
    Find {
        /// longitude of the point to check
        #[arg(long)]
        lng: f32,

        /// latitude of the point to check
        #[arg(long)]
        lat: f32,
    }
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
    Some(Commands::Populate { name, population, centroid }) => {
        spread_population(name.as_str(), *population, *centroid, &populator_config).expect("Population spreading failed");
    }
    Some(Commands::Find { lng, lat }) => {
        get_area_by_point(lat, lng, &populator_config).expect("Querying point info failed");
    }
    None => {}
}
}