use std::{
    env,
    fs::File,
    io::{self, BufReader},
};

use program_ingester::{input::Ingester, output::ProgramGraph};
use tracing_subscriber::layer::SubscriberExt;

fn main() -> anyhow::Result<()> {
    // Create a stdout logging layer
    let logger = tracing_subscriber::fmt::layer().with_writer(io::stderr);

    // Allow setting RUST_LOG level, or fallback to some level
    let fallback = "info";
    let env_filter = tracing_subscriber::EnvFilter::try_from_default_env()
        .or_else(|_| tracing_subscriber::EnvFilter::try_new(fallback))
        .unwrap();

    // Create a collector?
    let subscriber = tracing_subscriber::Registry::default()
        .with(logger) // .with(layer) Requires tracing_subscriber::layer::SubscriberExt
        .with(env_filter);

    // Initialize tracing
    tracing::subscriber::set_global_default(subscriber).expect("initialize tracing subscriber");

    // Read CLI arguments
    let args: Vec<String> = env::args().collect();

    // one day we can do if-let chains, like:
    // if let arg_len = args.len() && arg_len > 2 {
    // so that we can use the variable, as well as specify the condition for the if block
    if args.len() > 2 {
        tracing::warn!("additional arguments supplied and will be ignored");
    }

    // Build the ingester based on the specified file, or STDIN
    let ingester = if let Some(path) = args.get(1) {
        let reader = BufReader::new(File::open(path)?);
        Ingester::try_from(reader)?
    } else {
        let reader = BufReader::new(io::stdin());
        Ingester::try_from(reader)?
    };

    // Build the graph
    let graph = ProgramGraph::from(ingester.features);

    // Output the graph
    println!("{graph:#?}");

    Ok(())
}
