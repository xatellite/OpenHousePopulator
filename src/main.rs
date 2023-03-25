use clap::{Parser, Subcommand};
use config::Config;
use openhousepopulator::spread_population;

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
        inhabitants: u64,

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
            spread_population(file, inhabitants, centroid, &populator_config);
        }
        None => {}
    }
}
