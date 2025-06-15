use bitcoin::p2p::address::AddrV2;
use bitcoin::Network;
use bitcoin_peers_crawler::{CrawlerBuilder, CrawlerMessage, Peer, TransportPolicy};
use clap::{Parser, Subcommand};
use log::info;
use std::path::PathBuf;
use std::time::Instant;
use std::{error::Error, net::IpAddr};
use tokio::{
    select,
    time::{interval, Duration},
};

mod report;
mod stats;

use report::{CensusReport, OutputFormat};
use stats::NodeStats;

const USER_AGENT: &str = concat!("/census:", env!("CARGO_PKG_VERSION"), "/");

#[derive(Parser)]
#[command(
    name = "bitcoin-node-census",
    about = "Monitor and track bitcoin node feature adoption",
    version
)]
struct Cli {
    #[command(subcommand)]
    command: Commands,

    /// Logging level (trace, debug, info, warn, error)
    #[arg(long, default_value = "info", env = "RUST_LOG")]
    log_level: String,
}

#[derive(Subcommand)]
enum Commands {
    /// Run a census of the bitcoin network.
    Run {
        /// Seed node address (IP or hostname).
        #[arg(short = 'a', long, default_value = "seed.bitcoin.sipa.be")]
        address: String,
        /// Seed node port.
        #[arg(short = 'p', long, default_value = "8333")]
        port: u16,
        /// Maximum concurrent connections.
        #[arg(short, long, default_value = "32")]
        concurrent: usize,
        /// Output format.
        #[arg(short, long, value_enum, default_value = "json")]
        format: OutputFormat,
        /// Output file (stdout if not specified).
        #[arg(short, long)]
        output: Option<PathBuf>,
    },
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let cli = Cli::parse();

    let log_level = match cli.log_level.to_lowercase().as_str() {
        "error" => log::LevelFilter::Error,
        "warn" => log::LevelFilter::Warn,
        "info" => log::LevelFilter::Info,
        "debug" => log::LevelFilter::Debug,
        "trace" => log::LevelFilter::Trace,
        _ => log::LevelFilter::Info,
    };

    fern::Dispatch::new()
        .format(|out, message, record| {
            out.finish(format_args!(
                "[{}] {} - {}",
                record.level(),
                record.target(),
                message
            ))
        })
        .level(log_level)
        .chain(std::io::stderr())
        .apply()
        .unwrap();

    match cli.command {
        Commands::Run {
            address,
            port,
            concurrent,
            format,
            output,
        } => {
            run_census(address, port, concurrent, format, output).await?;
        }
    }

    Ok(())
}

async fn run_census(
    address: String,
    port: u16,
    concurrent: usize,
    format: OutputFormat,
    output: Option<PathBuf>,
) -> Result<(), Box<dyn Error>> {
    info!("BITCOIN NODE CENSUS");
    info!("Seed {address}:{port}, {concurrent} concurrent connections");

    let process_start = Instant::now();
    let mut stats = NodeStats::new();

    let crawler = CrawlerBuilder::new(Network::Bitcoin)
        .with_max_concurrent_tasks(concurrent)
        .with_transport_policy(TransportPolicy::V2Preferred)
        .with_protocol_version(70016)
        .with_user_agent(USER_AGENT)?
        .build();

    let ip_addr = tokio::net::lookup_host(format!("{address}:{port}"))
        .await?
        .next()
        .ok_or("Failed to resolve seed address")?
        .ip();

    let addr = match ip_addr {
        IpAddr::V4(ipv4) => AddrV2::Ipv4(ipv4),
        IpAddr::V6(ipv6) => AddrV2::Ipv6(ipv6),
    };

    let seed = Peer::new(addr, port);
    let mut receiver = crawler
        .crawl(seed)
        .await
        .map_err(|e| format!("Failed to start crawler: {e}"))?;

    let mut progress_interval = interval(Duration::from_secs(60));
    progress_interval.set_missed_tick_behavior(tokio::time::MissedTickBehavior::Skip);

    loop {
        select! {
            msg = receiver.recv() => {
                // Only care about listening nodes for stats, break when channel closed.
                match msg {
                    Some(CrawlerMessage::Listening(peer)) => {
                        stats.increment_contacted();
                        stats.add_node(peer);
                    }
                    Some(CrawlerMessage::NonListening(_)) => {
                        stats.increment_contacted();
                    }
                    None => break,
                }
            }
            _ = progress_interval.tick() => {
                let elapsed = process_start.elapsed().as_secs();
                stats.set_duration(elapsed);
                let report = CensusReport::from_stats(&stats);
                info!("{report}");
            }
        }
    }

    let duration = process_start.elapsed();
    info!(
        "Census complete: {} listening nodes out of {} contacted in {:.1} seconds",
        stats.total_nodes(),
        stats.total_contacted(),
        duration.as_secs_f64()
    );

    stats.set_duration(duration.as_secs());
    let report = CensusReport::from_stats(&stats);
    report.write(format, output)?;

    Ok(())
}
