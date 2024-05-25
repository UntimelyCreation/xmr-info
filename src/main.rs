use anyhow::anyhow;
use clap::{Parser, Subcommand};
use notify_rust::Notification;
use serde::Deserialize;
use serde_json::Value;
use std::{
    collections::HashMap,
    io::{self, Write},
    time::Duration,
};

#[derive(Debug, Parser)]
#[command(about)]
pub struct Args {
    #[clap(subcommand)]
    command: Command,
}

#[derive(Debug, Subcommand)]
enum Command {
    /// Calculate an XMR amount in another currency
    Convert(ConvertArgs),

    /// Send notification when P2Pool block is found
    Notify(NotifyBlockArgs),
}

#[derive(Debug, Parser)]
struct ConvertArgs {
    /// Currency code to convert to
    currency: String,

    /// Amount of XMR to convert
    #[arg(default_value_t = 1.0)]
    amount: f64,
}

#[derive(Debug, Parser)]
struct NotifyBlockArgs {
    /// Target P2Pool Mini
    #[clap(long, short)]
    mini: bool,

    /// Polling period, in seconds
    #[arg(default_value_t = 10)]
    period: u64,
}

#[tokio::main]
#[warn(clippy::pedantic)]
async fn main() -> Result<(), anyhow::Error> {
    let args = Args::parse();

    match args.command {
        Command::Convert(convert_args) => convert_xmr_amount(convert_args).await,
        Command::Notify(notify_args) => run_notify_block(notify_args).await,
    }
}

async fn convert_xmr_amount(args: ConvertArgs) -> Result<(), anyhow::Error> {
    let converted = get_xmr_price(&args.currency, &args.amount).await?;
    let output = format!("{} XMR = {} {}", args.amount, converted, args.currency);
    io::stdout().write_all(output.as_bytes())?;
    Ok(())
}

async fn get_xmr_price(curr: &str, amount: &f64) -> Result<f64, anyhow::Error> {
    let api_url = format!("https://min-api.cryptocompare.com/data/price?fsym=XMR&tsyms={curr}");
    let response = reqwest::get(api_url)
        .await?
        .json::<HashMap<String, Value>>()
        .await?;
    if let Some(base) = response.get(curr) {
        let converted = base.as_f64().ok_or(anyhow!("unable to fetch price"))? * amount;
        Ok(converted)
    } else {
        // TODO: Clean this up
        let err_msg = response
            .get("Message")
            .unwrap()
            .as_str()
            .unwrap()
            .to_string();
        Err(anyhow!(err_msg))
    }
}

#[derive(Debug, Clone, Deserialize)]
struct Block {
    hash: String,
    height: u32,
}

async fn run_notify_block(args: NotifyBlockArgs) -> Result<(), anyhow::Error> {
    if args.period == 0 {
        return Err(anyhow!("period must not be 0"));
    }

    let mut last_block_hash = get_current_block(args.mini).await?.hash;

    let mut interval = tokio::time::interval(Duration::from_secs(args.period));
    loop {
        interval.tick().await;
        let mut stdout = io::stdout();
        writeln!(stdout, "Fetching P2Pool block information...")?;
        let curr_block = get_current_block(args.mini).await?;
        if curr_block.hash != last_block_hash {
            last_block_hash = curr_block.hash;
            let curr_block_height = curr_block.height;
            Notification::new()
                .summary(&format!(
                    "P2Pool{} block found",
                    if args.mini { " Mini" } else { "" }
                ))
                .icon("monero")
                .body(&format!("New block found at height {curr_block_height}"))
                .show()?;
            writeln!(stdout, "Block found, notification sent!")?;
        }
    }
}

async fn get_current_block(mini: bool) -> Result<Block, anyhow::Error> {
    let api_url = format!(
        "https://p2pool.io{}/api/pool/blocks",
        if mini { "/mini" } else { "" }
    );
    let response = reqwest::get(api_url).await?.json::<Vec<Block>>().await?;
    response.first().ok_or(anyhow!("no blocks found")).cloned()
}
