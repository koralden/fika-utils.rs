use anyhow::{anyhow, Result};
use serde::{Deserialize, Serialize};
use tokio::fs;
use tracing::warn;

#[derive(Serialize, Deserialize, Debug, Default, Clone)]
#[allow(dead_code)]
pub struct KdaemonConfig {
    pub core: KCoreConfig,
    pub network: KNetworkConfig,
    pub por: KPorConfig,
    pub boss: KBossConfig,
    pub aws: Option<KAwsConfig>,
}

#[derive(Serialize, Deserialize, Debug, Default, Clone, PartialEq)]
#[allow(dead_code)]
pub struct KCoreConfig {
    pub wallet_address: Option<String>,
    pub mac_address: String,
    pub serial_number: String,
    pub sku: String,
    pub user_wallet: Option<String>,
}

impl KCoreConfig {
    pub async fn config_verify(&self) -> Result<()> {
        if self.wallet_address.is_none() {
            Err(anyhow!("ap-wallet-address invalid"))
        } else {
            Ok(())
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Default, Clone, PartialEq)]
#[allow(dead_code)]
pub struct KNetworkConfig {
    pub wan_type: u8,
    pub wan_username: Option<String>,
    pub wan_password: Option<String>,
    pub wifi_ssid: Option<String>,
    pub wifi_password: Option<String>,
    pub password_overwrite: Option<String>,
}

#[derive(Serialize, Deserialize, Debug, Default, Clone, PartialEq)]
#[allow(dead_code)]
pub struct KPorConfig {
    pub state: bool,
    pub nickname: Option<String>,
}

#[derive(Serialize, Deserialize, Debug, Default, Clone)]
#[allow(dead_code)]
pub struct KBossConfig {
    pub access_token: Option<String>,
    pub ap_access_token: Option<String>,
}

impl KBossConfig {
    pub async fn config_verify(&self) -> Result<()> {
        if self.access_token.is_none() {
            //return Err(anyhow!("access-token invalid"));
            warn!("access-token invalid");
        }

        if self.ap_access_token.is_none() {
            //Err(anyhow!("ap-access-token invalid"))
            warn!("ap-access-token invalid");
        }

        Ok(())
    }
}

impl KdaemonConfig {
    pub async fn build_from(path: &str) -> Result<Self> {
        let cfg = fs::read_to_string(path).await?;
        toml::from_str(&cfg).or_else(|e| Err(anyhow!(e)))
    }

    pub async fn config_verify(&self) -> Result<()> {
        self.core.config_verify().await?;
        self.boss.config_verify().await
    }
}

#[derive(Serialize, Deserialize, Debug, Default, Clone)]
pub struct KAwsConfig {
    pub auth_token: Option<String>,
}
