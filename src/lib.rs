use anyhow::{anyhow, Result};
use tokio::sync::{/*broadcast, Notify,*/ mpsc, oneshot};
use tracing::{debug, instrument};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};
use crate::kap_daemon::KdaemonConfig;
use crate::kap_rule::RuleConfig;
use std::path::PathBuf;
use tokio::time::Duration;
use serde::{Deserialize, Serialize};

pub mod activate;
pub mod aws_iot;
pub mod kap_daemon;
pub use self::activate::{activate, ActivateOpt};
pub mod misc;
pub mod web_api;
pub use self::web_api::{
    aws_web_cli, WebAwsOpt,
    boss_web_cli, WebBossOpt,
    curl_web_cli, CurlMethod,
};
#[cfg(feature = "boss-api")]
//pub use self::misc::{boss_tools, WebBossOpt};
pub use self::misc::{time_tools, TimeToolOpt};
#[cfg(feature = "ethers")]
pub use self::misc::{wallet_tools, WalletCommand};
pub mod kap_rule;

#[derive(Debug)]
#[allow(dead_code)]
pub enum DbCommand {
    Get {
        key: String,
        resp: oneshot::Sender<Option<String>>,
    },
    Set {
        key: String,
        val: String, //TODO Bytes
        resp: oneshot::Sender<Option<String>>,
    },
    Publish {
        key: String,
        val: String,
        resp: oneshot::Sender<Option<usize>>,
        //resp: mpsc::Sender<Option<String>>,
    },
    Lindex {
        key: String,
        idx: isize,
        resp: oneshot::Sender<Option<String>>,
    },
    Rpush {
        key: String,
        val: String,
        limit: usize,
    },
    /*AwsShadowPublish {
        key: String,
        val: String,
    },
    SubTaskNotify {
        topic: String,
        payload: String,
    },
    NotifySubscribe {
        key: String,
    },
    NotifySubscribeDone,*/
    Exit,
}

#[derive(Debug)]
#[allow(dead_code)]
pub enum SubscribeCmd {
    Notify { topic: String, msg: String },
    Exit,
}

#[derive(Deserialize, Serialize, Debug, Clone)]
#[allow(dead_code)]
pub struct RuleConfigTask {
    pub topic: String,
    pub path: PathBuf,
    pub start_at: Option<Duration>,
    pub period: Option<Duration>,
    pub db_publish: Option<bool>,
    pub db_set: Option<bool>,
    pub aws_publish: Option<bool>,
}

#[instrument(skip(chan_tx))]
pub async fn publish_message(
    chan_tx: &mpsc::Sender<DbCommand>,
    topic: String,
    payload: String,
) -> Result<()> {
    let (resp_tx, resp_rx) = oneshot::channel();

    chan_tx
        .send(DbCommand::Publish {
            key: topic.clone(),
            val: payload,
            resp: resp_tx,
        })
        .await?;

    let res = resp_rx.await;
    debug!(
        "[publish_task][publish][{}] transmit response {:?}",
        topic, res
    );

    Ok(())
}

#[instrument(skip(chan_tx))]
pub async fn set_message(
    chan_tx: mpsc::Sender<DbCommand>,
    topic: String,
    payload: String,
) -> Result<()> {
    let (resp_tx, resp_rx) = oneshot::channel();

    chan_tx
        .send(DbCommand::Set {
            key: topic.clone(),
            val: payload,
            resp: resp_tx,
        })
        .await?;

    let res = resp_rx.await;
    debug!(
        "[publish_task][publish][{}] transmit response {:?}",
        topic, res
    );

    Ok(())
}

pub fn setup_logging(log_level: &str) -> Result<()> {
    // See https://docs.rs/tracing for more info
    //tracing_subscriber::fmt::try_init()
    tracing_subscriber::registry()
        .with(tracing_subscriber::EnvFilter::new(
            std::env::var("RUST_LOG").unwrap_or_else(move |_| {
                format!("{},redis={},mio={}", log_level, log_level, log_level).into()
            }),
        ))
        .with(tracing_subscriber::fmt::layer())
        .init();
    Ok(())
}

pub async fn rule_config_load(
    rule_path: &str, cfg_path: Option<&str>
    ) -> Result<(RuleConfig, KdaemonConfig)> {
    let rule = RuleConfig::build_from(rule_path).await
        .map_err(|e| anyhow!("rule build from {} fail - {:?}", rule_path, e))?;

    let cfg_path = if let Some(path) = cfg_path {
        path
    } else {
        &rule.core.config
    };
    let cfg = KdaemonConfig::build_from(cfg_path).await
        .map_err(|e| anyhow!("cfg build from {} fail - {:?}",
                             cfg_path, e))?;

    Ok((rule, cfg))
}