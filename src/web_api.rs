use anyhow::{anyhow, Result};
use chrono::prelude::*;
use clap::{Args, Subcommand};
use colored_json::to_colored_json_auto;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::collections::HashMap;
use std::str::FromStr;
use thiserror::Error;
use tracing::error;

use crate::rule_config_load;
#[cfg(feature = "aws-cli")]
use crate::setup_logging;

#[derive(Error, Debug)]
pub enum CurlError {
    #[error("key-value {0} invalid")]
    KvFormat(String),
}

#[derive(Args, Debug)]
pub struct CurlKV {
    key: String,
    value: String,
}

impl FromStr for CurlKV {
    type Err = CurlError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if let Some((k, v)) = s.split_once(':') {
            Ok(Self {
                key: k.to_string(),
                value: v.to_string(),
            })
        } else {
            Err(CurlError::KvFormat(s.to_string()))
        }
    }
}

#[derive(Args, Debug)]
#[clap(about = "Curl-Post-Json")]
pub struct CurlPostJsonArgs {
    #[clap(short = 'H', long = "header", help = "KEY:ValUE(s)")]
    header: Option<Vec<CurlKV>>,

    #[clap(short = 'Q', long = "query", help = "KEY:ValUE(s)")]
    query: Option<Vec<CurlKV>>,

    #[clap(short = 'J', long = "json-data", help = "{...}")]
    json: Option<Value>,

    url: String,
}

#[derive(Args, Debug)]
#[clap(about = "Curl-Get")]
pub struct CurlGetArgs {
    #[clap(short = 'H', long = "header", help = "KEY:ValUE(s)")]
    header: Option<Vec<CurlKV>>,

    #[clap(short = 'Q', long = "query", help = "KEY:ValUE(s)")]
    query: Option<Vec<CurlKV>>,

    url: String,
}

#[derive(Args, Debug)]
#[clap(about = "Curl-Get-Json")]
pub struct CurlGetJsonArgs {
    #[clap(short = 'H', long = "header", help = "KEY:ValUE(s)")]
    header: Option<Vec<CurlKV>>,

    #[clap(short = 'Q', long = "query", help = "KEY:ValUE(s)")]
    query: Option<Vec<CurlKV>>,

    #[clap(short = 'J', long = "json-data", help = "{...}")]
    json: Option<Value>,

    url: String,
}

#[derive(Args, Debug)]
#[clap(about = "Curl-Post")]
pub struct CurlPostArgs {
    #[clap(short = 'H', long = "header", help = "KEY:ValUE(s)")]
    header: Option<Vec<CurlKV>>,

    #[clap(short = 'Q', long = "query", help = "KEY:ValUE(s)")]
    query: Option<Vec<CurlKV>>,

    #[clap(short = 'F', long = "form-data", help = "KEY:ValUE(s)")]
    form: Option<Vec<CurlKV>>,

    url: String,
}

#[derive(Debug, Subcommand)]
pub enum CurlMethod {
    Get(CurlGetArgs),
    GetJson(CurlGetJsonArgs),
    Post(CurlPostArgs),
    PostJson(CurlPostJsonArgs),
}

#[derive(Debug)]
pub enum CurlResponse {
    TextFmt(String),
    JsonFmt(Value),
}

#[allow(dead_code)]
async fn curl_web_api(method: CurlMethod) -> Result<CurlResponse> {
    let client = reqwest::Client::new();
    match method {
        CurlMethod::Get(args) => {
            let mut req = client.get(&format!("{}", &args.url));

            req = if let Some(hs) = args.header {
                for h in hs {
                    req = req.header(h.key, h.value);
                }
                req
            } else {
                req
            };

            req = if let Some(qs) = args.query {
                for q in qs {
                    req = req.query(&[(q.key, q.value)])
                }
                req
            } else {
                req
            };

            req.send()
                .await?
                .text()
                .await
                .map(|r| CurlResponse::TextFmt(r))
                .map_err(|e| anyhow!("{:?}", e))
        }
        CurlMethod::GetJson(args) => {
            let mut req = client.get(&format!("{}", &args.url));

            req = if let Some(hs) = args.header {
                for h in hs {
                    req = req.header(h.key, h.value);
                }
                req
            } else {
                req
            };

            req = if let Some(qs) = args.query {
                for q in qs {
                    req = req.query(&[(q.key, q.value)])
                }
                req
            } else {
                req
            };

            if let Some(js) = args.json {
                req.json(&js)
            } else {
                req
            }
            .send()
            .await?
            .json::<Value>()
            .await
            .map(|r| CurlResponse::JsonFmt(r))
            .map_err(|e| anyhow!("{:?}", e))
        }
        CurlMethod::Post(args) => {
            let mut req = client.post(&format!("{}", &args.url));

            req = if let Some(hs) = args.header {
                for h in hs {
                    req = req.header(h.key, h.value);
                }
                req
            } else {
                req
            };

            req = if let Some(qs) = args.query {
                for q in qs {
                    req = req.query(&[(q.key, q.value)])
                }
                req
            } else {
                req
            };

            req = if let Some(fs) = args.form {
                let mut map: HashMap<String, String> = HashMap::new();
                for f in fs {
                    map.insert(f.key, f.value);
                }
                req.form(&map)
            } else {
                req
            };

            req.send()
                .await?
                .text()
                .await
                .map(|r| CurlResponse::TextFmt(r))
                .map_err(|e| anyhow!("{:?}", e))
        }
        CurlMethod::PostJson(args) => {
            let mut req = client.post(&format!("{}", &args.url));

            req = if let Some(hs) = args.header {
                for h in hs {
                    req = req.header(h.key, h.value);
                }
                req
            } else {
                req
            };

            req = if let Some(qs) = args.query {
                for q in qs {
                    req = req.query(&[(q.key, q.value)])
                }
                req
            } else {
                req
            };

            if let Some(js) = args.json {
                req.json(&js)
            } else {
                req
            }
            .send()
            .await?
            .json::<Value>()
            .await
            .map(|r| CurlResponse::JsonFmt(r))
            .map_err(|e| anyhow!("{:?}", e))
        }
    }
}

pub async fn curl_web_cli(method: CurlMethod) -> Result<()> {
    let resp = curl_web_api(method).await?;
    match resp {
        CurlResponse::TextFmt(s) => println!("{s}"),
        CurlResponse::JsonFmt(j) => println!("{}", to_colored_json_auto(&j)?),
    }
    Ok(())
}

#[derive(Args, Debug)]
pub struct ApWalletArg {
    #[clap(help = r#"{"who": $who, "where": $where, "comment": $comment }"#)]
    json: Value,

    #[clap(long = "path", default_value = "v0/device/get_eth_wallet")]
    path: String,
}

#[derive(Args, Debug)]
pub struct ApHcsArg {
    #[clap(help = r#"{"ap_wallet":$wallet,"hcs_token":$hcs_token,"hash":$hash}"#)]
    json: Value,

    #[clap(long = "path", default_value = "v0/ap/hcs")]
    path: String,
}

#[derive(Args, Debug)]
pub struct ApTokenArg {
    #[clap(long = "path", default_value = "v0/ap/ap_token")]
    path: String,
}

#[derive(Args, Debug)]
pub struct OtpArg {
    #[clap(long = "path", default_value = "v0/ap/otp")]
    pub path: String,
}

#[derive(Args, Debug)]
pub struct HcsArg {
    #[clap(long = "path", default_value = "v0/hcs/pair")]
    path: String,
}

#[derive(Args, Debug)]
pub struct ApInfoArg {
    #[clap(long = "path", default_value = "v0/ap/info")]
    pub path: String,
}

#[derive(Subcommand, Debug)]
#[clap(about = "Web/Boss")]
pub enum WebBossPath {
    GetApToken(ApTokenArg),
    GetOtp(OtpArg),
    GetHcs(HcsArg),
    GetApInfo(ApInfoArg),
    GetApWallet(ApWalletArg),
    PostApHcs(ApHcsArg),
}

#[derive(Args, Debug)]
#[clap(about = "Boss web api")]
pub struct WebBossOpt {
    #[clap(subcommand)]
    class: WebBossPath,

    #[clap(
        short = 'r',
        long = "rule",
        default_value = "/etc/fika_manager/rule.toml"
    )]
    rule: String,

    #[clap(short = 'u', long = "root-url")]
    root: Option<String>,

    #[clap(short = 'r', long = "access-region")]
    access_region: Option<String>,

    #[clap(short = 't', long = "ap-access-token")]
    access_token: Option<String>,

    #[clap(short = 'w', long = "ap-wallet")]
    wallet: Option<String>,
}

#[cfg(feature = "boss-api")]
#[allow(dead_code)]
pub async fn boss_web_api(
    wallet: Option<String>,
    root_url: String,
    region: String,
    token: Option<String>,
    class: WebBossPath,
) -> Result<serde_json::Value> {
    match class {
        WebBossPath::GetApToken(arg) => {
            let wallet = if let Some(w) = wallet {
                w
            } else {
                return Err(anyhow::anyhow!("wallet-address invalid"));
            };

            match curl_web_api(CurlMethod::GetJson(CurlGetJsonArgs {
                header: Some(vec![CurlKV {
                    key: "ACCESSTOKEN".to_string(),
                    value: region,
                }]),
                query: Some(vec![CurlKV {
                    key: "ap_wallet".to_string(),
                    value: wallet,
                }]),
                json: None,
                url: format!("{}/{}", root_url, &arg.path),
            }))
            .await?
            {
                CurlResponse::JsonFmt(response) => {
                    if response["code"] == 200 {
                        Ok(response)
                    } else {
                        Err(anyhow::anyhow!(
                            "{} [{}]",
                            response["message"],
                            response["code"]
                        ))
                    }
                }
                CurlResponse::TextFmt(s) => Err(anyhow::anyhow!("text format - {s}")),
            }
        }
        WebBossPath::PostApHcs(map) => {
            if token.is_none() {
                error!("[kap][boss] ap-acess-token not exist");
                return Err(anyhow!("[kap][boss] ap-acess-token not exist"));
            }

            let wallet = if let Some(w) = wallet {
                w
            } else {
                return Err(anyhow::anyhow!("wallet-address invalid"));
            };

            match curl_web_api(CurlMethod::PostJson(CurlPostJsonArgs {
                header: Some(vec![
                    CurlKV {
                        key: "ACCESSTOKEN".to_string(),
                        value: region,
                    },
                    CurlKV {
                        key: "ACCESSTOKEN-AP".to_string(),
                        value: token.unwrap(),
                    },
                ]),
                query: Some(vec![CurlKV {
                    key: "ap_wallet".to_string(),
                    value: wallet,
                }]),
                json: Some(map.json),
                url: format!("{}/{}", root_url, &map.path),
            }))
            .await?
            {
                CurlResponse::JsonFmt(response) => {
                    if response["code"] == 200 {
                        Ok(response["data"].clone())
                    } else {
                        Err(anyhow::anyhow!(
                            "{} [{}]",
                            response["message"],
                            response["code"]
                        ))
                    }
                }
                CurlResponse::TextFmt(s) => Err(anyhow::anyhow!("text format - {s}")),
            }
        }
        WebBossPath::GetOtp(arg) => {
            if token.is_none() {
                error!("[kap][boss] ap-acess-token not exist");
                return Err(anyhow!("[kap][boss] ap-acess-token not exist"));
            }

            let wallet = if let Some(w) = wallet {
                w
            } else {
                return Err(anyhow::anyhow!("wallet-address invalid"));
            };

            match curl_web_api(CurlMethod::GetJson(CurlGetJsonArgs {
                header: Some(vec![
                    CurlKV {
                        key: "ACCESSTOKEN".to_string(),
                        value: region,
                    },
                    CurlKV {
                        key: "ACCESSTOKEN-AP".to_string(),
                        value: token.unwrap(),
                    },
                ]),
                query: Some(vec![CurlKV {
                    key: "ap_wallet".to_string(),
                    value: wallet,
                }]),
                json: None,
                url: format!("{}/{}", root_url, &arg.path),
            }))
            .await?
            {
                CurlResponse::JsonFmt(response) => {
                    if response["code"] == 200 {
                        Ok(response)
                    } else {
                        Err(anyhow::anyhow!(
                            "{} [{}]",
                            response["message"],
                            response["code"]
                        ))
                    }
                }
                CurlResponse::TextFmt(s) => Err(anyhow::anyhow!("text format - {s}")),
            }
        }
        WebBossPath::GetHcs(arg) => {
            if token.is_none() {
                error!("[kap][boss] ap-acess-token not exist");
                return Err(anyhow!("[kap][boss] ap-acess-token not exist"));
            }

            let wallet = if let Some(w) = wallet {
                w
            } else {
                return Err(anyhow::anyhow!("wallet-address invalid"));
            };

            match curl_web_api(CurlMethod::GetJson(CurlGetJsonArgs {
                header: Some(vec![
                    CurlKV {
                        key: "ACCESSTOKEN".to_string(),
                        value: region,
                    },
                    CurlKV {
                        key: "ACCESSTOKEN-AP".to_string(),
                        value: token.unwrap(),
                    },
                ]),
                query: Some(vec![CurlKV {
                    key: "ap_wallet".to_string(),
                    value: wallet,
                }]),
                json: None,
                url: format!("{}/{}", root_url, &arg.path),
            }))
            .await?
            {
                CurlResponse::JsonFmt(response) => {
                    if response["code"] == 200 {
                        Ok(response["hcs"].clone())
                    } else {
                        return Err(anyhow::anyhow!(
                            "{} [{}]",
                            response["message"],
                            response["code"]
                        ));
                    }
                }
                CurlResponse::TextFmt(s) => return Err(anyhow::anyhow!("text format - {s}")),
            }
        }
        WebBossPath::GetApInfo(arg) => {
            if token.is_none() {
                error!("[kap][boss] ap-acess-token not exist");
                return Err(anyhow!("[kap][boss] ap-acess-token not exist"));
            }

            let path = arg.path;

            let wallet = if let Some(w) = wallet {
                w
            } else {
                return Err(anyhow::anyhow!("wallet-address invalid"));
            };

            match curl_web_api(CurlMethod::GetJson(CurlGetJsonArgs {
                header: Some(vec![
                    CurlKV {
                        key: "ACCESSTOKEN".to_string(),
                        value: region,
                    },
                    CurlKV {
                        key: "ACCESSTOKEN-AP".to_string(),
                        value: token.unwrap(),
                    },
                ]),
                query: None,
                json: Some(json!({ "ap_wallet": wallet })),
                url: format!("{}/{}", root_url, &path),
            }))
            .await?
            {
                CurlResponse::JsonFmt(response) => {
                    if response["code"] == 200 {
                        Ok(response["data"].clone())
                    } else {
                        Err(anyhow::anyhow!(
                            "{} [{}]",
                            response["message"],
                            response["code"]
                        ))
                    }
                }
                CurlResponse::TextFmt(s) => Err(anyhow::anyhow!("text format - {s}")),
            }
        }
        WebBossPath::GetApWallet(map) => {
            match curl_web_api(CurlMethod::GetJson(CurlGetJsonArgs {
                header: Some(vec![CurlKV {
                    key: "ACCESSTOKEN".to_string(),
                    value: region,
                }]),
                query: None,
                json: Some(map.json),
                url: format!("{}/{}", root_url, &map.path),
            }))
            .await?
            {
                CurlResponse::JsonFmt(response) => {
                    if response["code"] == 200 {
                        Ok(response["data"].clone())
                    } else {
                        Err(anyhow::anyhow!(
                            "{} [{}]",
                            response["message"],
                            response["code"]
                        ))
                    }
                }
                CurlResponse::TextFmt(s) => Err(anyhow::anyhow!("text format - {s}")),
            }
        }
    }
}

#[cfg(feature = "boss-api")]
#[allow(dead_code)]
pub async fn boss_web_cli(opt: WebBossOpt) -> Result<()> {
    let (rule, cfg) = rule_config_load(&opt.rule, None).await?;

    let core = cfg.core;
    let boss = cfg.boss;

    let root_url = if let Some(root) = opt.root {
        root
    } else {
        rule.boss.root_url.unwrap()
    };

    let region = if let Some(region) = opt.access_region {
        region
    } else {
        boss.access_token.unwrap()
    };

    let token = if let Some(token) = opt.access_token {
        Some(token)
    } else {
        boss.ap_access_token
    };

    let wallet = if let Some(wallet) = opt.wallet {
        Some(wallet)
    } else {
        core.wallet_address
    };

    let resp = boss_web_api(wallet, root_url, region, token, opt.class).await?;
    println!("{}", to_colored_json_auto(&resp)?);
    Ok(())
}

#[derive(Args, Debug)]
pub struct DeviceArgs {
    #[clap(long = "online")]
    online: bool,

    wallet: Option<String>,

    #[clap(
        short = 'd',
        long = "devic-path",
        default_value = "prod/api/v1/devices"
    )]
    device_path: String,
}

#[derive(Subcommand, Debug)]
#[clap(about = "Web/AWS")]
pub enum WebAwsPath {
    GetDevice(DeviceArgs),
}

#[derive(Args, Debug)]
#[clap(about = "AWS/IOT web api")]
pub struct WebAwsOpt {
    #[clap(subcommand)]
    class: WebAwsPath,

    /*#[clap(short = 'r', long = "rule", default_value = "/etc/fika_manager/rule.toml")]
    rule: String,*/
    #[clap(long = "log-level", default_value = "info")]
    log_level: String,

    #[clap(short = 'u', long = "root-url")]
    root_url: Option<String>,

    #[clap(short = 'a', long = "auth-token")]
    auth_token: Option<String>,

    #[clap(
        short = 'r',
        long = "rule",
        default_value = "/etc/fika_manager/rule.toml"
    )]
    rule: String,
}

#[derive(Debug, Serialize, Deserialize)]
struct AwsDeviceEntry {
    device: Option<String>,
    owner: Option<String>,
    systime_time: DateTime<Utc>,
}

#[derive(Debug, Serialize, Deserialize)]
struct AwsDeviceList {
    data: Vec<AwsDeviceEntry>,
}

#[cfg(feature = "aws-cli")]
pub async fn aws_web_api(root_url: &str, auth_token: &str, class: WebAwsPath) -> Result<()> {
    match class {
        WebAwsPath::GetDevice(state) => {
            match curl_web_api(CurlMethod::GetJson(CurlGetJsonArgs {
                header: Some(vec![CurlKV {
                    key: "authorizationToken".to_string(),
                    value: auth_token.to_string(),
                }]),
                query: if state.online {
                    Some(vec![CurlKV {
                        key: "state".to_string(),
                        value: "online".to_string(),
                    }])
                } else {
                    None
                },
                json: None,
                url: format!("{}/{}", root_url, &state.device_path),
            }))
            .await?
            {
                CurlResponse::JsonFmt(response) => {
                    let response: AwsDeviceList = serde_json::from_value(response).unwrap();
                    let show = if let Some(ref wallet) = state.wallet {
                        let got = response
                            .data
                            .iter()
                            .filter(|i| {
                                if let Some(ref w) = i.device {
                                    w == wallet
                                } else {
                                    false
                                }
                            })
                            .collect::<Vec<&AwsDeviceEntry>>();
                        serde_json::to_value(&got)?
                    } else {
                        serde_json::to_value(&response.data)?
                    };
                    println!("{}", to_colored_json_auto(&show)?);
                }
                CurlResponse::TextFmt(s) => return Err(anyhow::anyhow!("text format - {s}")),
            }
        }
    }

    Ok(())
}

#[cfg(feature = "aws-cli")]
pub async fn aws_web_cli(opt: WebAwsOpt) -> Result<()> {
    setup_logging(&opt.log_level)?;

    let (rule, cfg) = rule_config_load(&opt.rule, None).await?;

    let root_url = if let Some(root) = opt.root_url {
        root
    } else {
        rule.aws.root_url.unwrap()
    };
    let auth_token = if let Some(token) = opt.auth_token {
        token
    } else {
        cfg.aws
            .expect("aws section nonexist")
            .auth_token
            .expect("auth-token nonexist")
    };

    aws_web_api(&root_url, &auth_token, opt.class).await
}

pub fn web_full_url(url: &str, path: &str, query: &Vec<(&str, &str)>) -> Result<String> {
    let url = reqwest::Url::parse_with_params(&format!("{}/{}", url, path), query)?;

    Ok(url.into())
}
