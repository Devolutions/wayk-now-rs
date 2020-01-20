use std::{net::SocketAddr, str::FromStr};
use structopt::StructOpt;
use wayk_proto::message::{AuthType, ChannelName, NowCapset};

#[derive(StructOpt, Debug)]
#[structopt(author, about)]

pub struct Cli {
    pub addr: SocketAddr,

    #[structopt(short, long)]
    /// Enable verbose logging for debug purpose
    pub debug: bool,

    #[structopt(short, long, env = "WAYK_CLI_AUTH")]
    /// Authentication method to use. Available: PFP, None
    ///
    /// PFP: `pfp:<friendly_name>,<friendly_text>`
    ///
    /// None: `none`
    ///
    /// Note that case is ignored for the method name (both "PFP" and "pfp" are accepted).
    pub auth: AuthConfig,

    #[structopt(short, long)]
    /// Configure chat preferences. `<friendly_name>[,<status_text>]`
    pub chat_config: Option<ChatConfig>,

    #[structopt(long = "on-sync")]
    /// Message to send on synchronisation
    pub on_sync_message: Option<String>,

    #[structopt(long)]
    /// Text to put into server clipboard
    pub on_clipboard_ready: Option<String>,
}

#[derive(Debug, Clone)]
pub struct PFPConfig {
    pub friendly_name: String,
    pub friendly_text: String,
}

#[derive(Debug, Clone)]
pub enum AuthConfig {
    PFP(PFPConfig),
    None,
}

impl AuthConfig {
    pub fn available_methods() -> &'static str {
        "PFP, None"
    }

    pub fn auth_type(&self) -> AuthType {
        match self {
            AuthConfig::PFP(_) => AuthType::PFP,
            AuthConfig::None => AuthType::None,
        }
    }
}

impl FromStr for AuthConfig {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let s = s.trim();

        let (method_name, body) = {
            let end_token_pos = s.find(':');
            if let Some(pos) = end_token_pos {
                (s[0..pos].trim().to_string(), &s[pos + 1..])
            } else {
                (s.to_string(), "")
            }
        };

        match method_name.to_lowercase().as_str() {
            "pfp" => {
                let args = body.split(',').collect::<Vec<&str>>();
                if args.len() < 2 {
                    Err(format!(
                        "Invalid PFP arguments in `{}`. Syntax is `PFP:<friendly_name>,<friendly_text>`",
                        s
                    ))
                } else {
                    Ok(Self::PFP(PFPConfig {
                        friendly_name: args[0].trim().to_string(),
                        friendly_text: args[1].trim().to_string(),
                    }))
                }
            }
            "none" => Ok(Self::None),
            unknown_method => Err(format!(
                "Unknown authentication method `{}`. Available methods: {}",
                unknown_method,
                AuthConfig::available_methods()
            )),
        }
    }
}

#[derive(Debug, Clone)]
pub struct ChatConfig {
    pub friendly_name: String,
    pub status_text: Option<String>,
}

impl FromStr for ChatConfig {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let s = s.trim();
        let mut args = s.split(',');
        let friendly_name = if let Some(name) = args.next() {
            name.to_owned()
        } else {
            return Err("friendly name missing".into());
        };
        let status_text = args.next().map(|s| s.to_owned());

        Ok(Self {
            friendly_name,
            status_text,
        })
    }
}

pub fn configure_capabilities() -> Vec<NowCapset<'static>> {
    use wayk_proto::message::{connection_sequence::capabilities::*, now_messages::MouseMode};

    vec![
        NowCapset::Transport(TransportCapset::default()),
        NowCapset::Update(UpdateCapset::new_with_supported_codecs(vec![
            NowCodecDef::new_with_flags(Codec::JPEG, 0x0000_0001),
        ])),
        NowCapset::License(LicenseCapset {
            flags: LicenseCapsetFlags::new_empty(),
        }),
        NowCapset::Mouse(MouseCapset::new(MouseMode::Primary, MouseCapsetFlags::new_empty())),
    ]
}

pub fn configure_available_auth_types() -> Vec<AuthType> {
    vec![AuthType::None, AuthType::PFP]
}

pub fn configure_channels_to_open() -> Vec<ChannelName> {
    vec![ChannelName::Clipboard, ChannelName::Chat]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_pfp_method() {
        let pfp_methods = vec![
            AuthConfig::from_str("PFP : Joe , It's me\t").unwrap(),
            AuthConfig::from_str("\tPFP: Joe    ,It's me").unwrap(),
            AuthConfig::from_str("   PFP:     Joe,It's me     ").unwrap(),
        ];

        for method in pfp_methods {
            if let AuthConfig::PFP(conf) = method {
                assert_eq!(conf.friendly_name, "Joe");
                assert_eq!(conf.friendly_text, "It's me");
            } else {
                panic!("parsed wrong auth method");
            }
        }
    }
}
