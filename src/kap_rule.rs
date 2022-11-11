use anyhow::{anyhow, Result};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use tokio::fs;
use tokio::time::Duration;

use crate::RuleConfigTask;
#[cfg(feature = "aws-iot")]
use {
    crate::aws_iot::{RuleAwsIotDedicatedConfig, RuleAwsIotProvisionConfig},
    fastrand,
    std::iter::repeat_with,
};

#[derive(Deserialize, Serialize, Debug)]
#[allow(dead_code)]
pub struct RuleConfig {
    pub core: RuleConfigCore,
    pub boss: RuleConfigBoss,
    pub subscribe: Option<Vec<RuleConfigSubscribe>>,
    pub task: Option<Vec<RuleConfigTask>>,
    pub honest: Option<RuleHonestConfig>,
    pub aws: RuleAwsIotConfig,
}

impl RuleConfig {
    fn mirrow_default(mut self) -> Result<Self> {
        self.core.mirrow_default()?;
        self.boss.mirrow_default()?;
        self.aws.mirrow_default()?;

        Ok(self)
    }

    pub async fn build_from(path: &str) -> Result<Self> {
        let cfg = fs::read_to_string(path).await?;
        match toml::from_str::<Self>(&cfg) {
            Ok(r) => Self::mirrow_default(r),
            Err(e) => Err(anyhow!("rule format invalid - {:?}", e)),
        }
    }
}

#[derive(Deserialize, Serialize, Debug)]
#[allow(dead_code)]
pub struct RuleConfigCore {
    pub thirdparty: String,
    pub database: Option<String>,
    pub config: String,
}

impl RuleConfigCore {
    fn mirrow_default(&mut self) -> Result<()> {
        let def: Self = Default::default();

        if self.database.is_none() {
            self.database = def.database;
        }

        Ok(())
    }
}

impl Default for RuleConfigCore {
    fn default() -> Self {
        Self {
            thirdparty: "longdong2".to_string(),
            database: Some("redis://127.0.0.1:6379".to_string()),
            config: "/userdata/kdaemon.toml".to_string(),
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[allow(dead_code)]
pub struct RuleConfigBoss {
    pub root_url: Option<String>,
    pub otp_path: Option<String>,
    pub ap_token_path: Option<String>,
    pub hcs_path: Option<String>,
    pub ap_hcs_path: Option<String>,
    pub ap_info_path: Option<String>,
}

impl RuleConfigBoss {
    fn mirrow_default(&mut self) -> Result<()> {
        let def: Self = Default::default();

        if self.root_url.is_none() {
            self.root_url = def.root_url;
        }
        if self.otp_path.is_none() {
            self.otp_path = def.otp_path;
        }
        if self.ap_token_path.is_none() {
            self.ap_token_path = def.ap_token_path;
        }
        if self.hcs_path.is_none() {
            self.hcs_path = def.hcs_path;
        }
        if self.ap_hcs_path.is_none() {
            self.ap_hcs_path = def.ap_hcs_path;
        }
        if self.ap_info_path.is_none() {
            self.ap_info_path = def.ap_info_path;
        }

        Ok(())
    }
}

impl Default for RuleConfigBoss {
    fn default() -> Self {
        Self {
            root_url: Some("https://oss-api.k36588.info".to_string()),
            otp_path: Some("v0/ap/otp".to_string()),
            ap_token_path: Some("v0/ap/ap_token".to_string()),
            hcs_path: Some("v0/hcs/pair".to_string()),
            ap_hcs_path: Some("v0/ap/hcs".to_string()),
            ap_info_path: Some("v0/ap/info".to_string()),
        }
    }
}

#[derive(Deserialize, Serialize, Debug, Clone)]
#[allow(dead_code)]
pub struct RuleConfigSubscribe {
    pub topic: String,
    pub path: PathBuf,
}

#[derive(Deserialize, Serialize, Debug)]
#[allow(dead_code)]
pub struct RuleHonestConfig {
    pub ok_cycle: Duration,
    pub fail_cycle: Duration,
    pub path: PathBuf,
    pub disable: Option<bool>,
}

#[derive(Deserialize, Serialize, Debug, Clone)]
#[allow(dead_code)]
pub struct RuleAwsIotConfig {
    #[cfg(feature = "aws-cli")]
    pub root_url: Option<String>,
    #[cfg(feature = "aws-cli")]
    pub device_path: Option<String>,

    #[cfg(feature = "aws-iot")]
    pub endpoint: Option<String>,
    #[cfg(feature = "aws-iot")]
    pub port: Option<u32>,

    #[cfg(feature = "aws-iot")]
    pub provision: Option<RuleAwsIotProvisionConfig>,
    #[cfg(feature = "aws-iot")]
    pub dedicated: RuleAwsIotDedicatedConfig,
}

impl RuleAwsIotConfig {
    pub async fn config_verify(&self) -> Result<()> {
        #[cfg(feature = "aws-iot")]
        if self.endpoint.is_none() {
            return Err(anyhow!("rule/aws/cfg endpoint invalid"));
        }
        #[cfg(feature = "aws-iot")]
        if self.port.is_none() {
            return Err(anyhow!("rule/aws/cfg port invalid"));
        }

        #[cfg(feature = "aws-iot")]
        self.dedicated.config_verify().await?;

        Ok(())
    }

    fn mirrow_default(&mut self) -> Result<()> {
        #[cfg(any(feature = "aws-cli", feature = "aws-iot"))]
        let def: Self = Default::default();

        #[cfg(feature = "aws-cli")]
        if self.root_url.is_none() {
            self.root_url = def.root_url;
        }
        #[cfg(feature = "aws-cli")]
        if self.device_path.is_none() {
            self.device_path = def.device_path;
        }
        #[cfg(feature = "aws-iot")]
        if self.endpoint.is_none() {
            self.endpoint = def.endpoint;
        }
        #[cfg(feature = "aws-iot")]
        if self.port.is_none() {
            self.port = def.port;
        }

        Ok(())
    }

    #[cfg(feature = "aws-iot")]
    pub fn thing_name(&self, postfix: &str) -> Result<String> {
        let thing = if let Some(ref thing) = self.dedicated.thing {
            thing.clone()
        } else {
            let prefix = if let Some(ref prov) = self.provision {
                &prov.thing_prefix
            } else {
                "Fake"
            };

            format!("{}_{}", prefix, postfix.to_lowercase().replace(":", ""))
        };
        Ok(thing)
    }

    #[cfg(feature = "aws-iot")]
    pub fn client_id(&self) -> Result<String> {
        Ok(repeat_with(fastrand::alphanumeric).take(5).collect())
    }
}

#[cfg(any(feature = "aws-cli", feature = "aws-iot"))]
impl Default for RuleAwsIotConfig {
    fn default() -> Self {
        Self {
            #[cfg(feature = "aws-iot")]
            endpoint: Some("a2dl0okey4lms3-ats.iot.ap-northeast-1.amazonaws.com".to_string()),
            #[cfg(feature = "aws-iot")]
            port: Some(8883),
            #[cfg(feature = "aws-cli")]
            root_url: Some(
                "https://i76cqmiru3.execute-api.ap-northeast-1.amazonaws.com".to_string(),
            ),
            #[cfg(feature = "aws-cli")]
            device_path: Some("prod/api/v1/devices".to_string()),

            #[cfg(feature = "aws-iot")]
            provision: None,
            #[cfg(feature = "aws-iot")]
            dedicated: RuleAwsIotDedicatedConfig::default(),
        }
    }
}
