use anyhow::Result;
use clap::{Args, Subcommand};
use serde_json::Value;
use tracing::{debug, instrument};
//use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

#[cfg(feature = "wallet")]
use ethers::prelude::*;

use chrono::prelude::*;

use crate::setup_logging;

#[derive(Args, Debug)]
struct ApWalletOpt {
    #[clap(help = r#"{"who": $who, "where": $where, "comment": $comment }"#)]
    json: Value,
}

#[derive(Args, Debug)]
struct ApHcsOpt {
    #[clap(help = r#"{"ap_wallet":$wallet,"hcs_token":$hcs_token,"hash":$hash}"#)]
    json: Value,
}

#[derive(Args, Debug, Clone)]
#[clap(about = "Generate Wallet")]
pub struct GenerateOpt {
    #[clap(short = 'o', long = "output")]
    output: Option<String>,
}

#[derive(Subcommand, Debug)]
pub enum WalletCommand {
    Generate(GenerateOpt),
    //Transact(TransactOpt),
    //Balance(BalanceOpt),
}

#[derive(Args, Debug)]
#[clap(about = "Timestamp now")]
pub struct TimestampOpt {
    timestamp: DateTime<Utc>,
}

#[derive(Args, Debug)]
#[clap(about = "FIKA Time Toolset")]
pub struct TimeToolOpt {
    #[clap(subcommand)]
    commands: TimeToolCommand,

    #[clap(short = 'l', long = "log-level", default_value = "info")]
    log_level: String,
}

#[derive(Subcommand, Debug)]
enum TimeToolCommand {
    Timestamp(TimestampOpt),
    Rfc3339,
}

#[instrument(name = "timestamp")]
async fn do_timestamp(t: DateTime<Utc>) -> Result<()> {
    debug!("DateTime - {:?} to Timestamp - {}", t, t.timestamp());
    println!("{}", t.timestamp());
    Ok(())
}

#[instrument(name = "rfc3339")]
async fn do_rfc3339() -> Result<()> {
    let now = Utc::now();
    println!("{}", now.to_rfc3339_opts(SecondsFormat::Secs, false));
    Ok(())
}

#[cfg(feature = "wallet")]
#[instrument(name = "wallet")]
pub async fn wallet_tools(w: WalletCommand) -> Result<()> {
    match w {
        WalletCommand::Generate(_cfg) => {
            let wallet = LocalWallet::new(&mut rand::thread_rng());
            println!("{:?}", wallet.address());
        }
    }
    Ok(())
}

//#[tokio::main]
pub async fn time_tools(opt: TimeToolOpt) -> Result<()> {
    setup_logging(&opt.log_level)?;

    match opt.commands {
        TimeToolCommand::Timestamp(t) => {
            do_timestamp(t.timestamp).await?;
        }
        TimeToolCommand::Rfc3339 => {
            do_rfc3339().await?;
        }
    }

    Ok(())
}
/*#[tokio::test]
async fn test_toml_duration() {
    let cp = ConfigTask {
        topic: String::from("test"),
        path: PathBuf::from("/tmp/test.sh"),
        start_at: Some(Duration::from_secs(1)),
        period: Some(Duration::from_secs(10)),
    };

    let toml = toml::to_string(&cp);
    assert_eq!(toml, Ok(String::from("hello")));
}*/
