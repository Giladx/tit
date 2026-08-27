use clap::{Parser, Subcommand};
use uuid::Uuid;
use base64::{Engine as _, engine::general_purpose::STANDARD};
use sha2::{Sha256, Sha512, Digest};
use std::str::FromStr;
use ipnet::Ipv4Net;
use serde_json::Value as JsonValue;
use rand::Rng;

#[derive(Parser)]
#[command(name = "tit", version, about = "Terminal UI toolbox and headless agent tools")]
pub struct Cli {
    #[command(subcommand)]
    pub command: Option<Commands>,
}

#[derive(Subcommand)]
pub enum Commands {
    /// Generate UUIDs
    Uuid {
        #[arg(short, long, default_value_t = 1)]
        count: usize,
    },
    /// Base64 Encode/Decode
    Base64 {
        #[arg(short, long)]
        decode: bool,
        text: String,
    },
    /// URL Encode/Decode
    Urlencode {
        #[arg(short, long)]
        decode: bool,
        text: String,
    },
    /// HTML Entities Encode/Decode
    HtmlEntities {
        #[arg(short, long)]
        decode: bool,
        text: String,
    },
    /// Generate Hashes (MD5, SHA256, SHA512)
    Hash {
        text: String,
    },
    /// Decode JWT token
    Jwt {
        token: String,
    },
    /// Text Statistics
    Stats {
        text: String,
    },
    /// IPv4 Subnet Calculator
    Ipv4 {
        cidr: String,
    },
    /// MAC Address Generator
    Mac {
        #[arg(short, long, default_value_t = 1)]
        count: usize,
    },
    /// JSON/YAML Converter
    Yaml2json {
        text: String,
    },
    Json2yaml {
        text: String,
    },
    /// Number Base Converter
    NumberBase {
        #[arg(short, long)]
        from: String,
        value: String,
    },
}

pub fn handle_cli(cmd: Commands) -> anyhow::Result<()> {
    match cmd {
        Commands::Uuid { count } => {
            for _ in 0..count {
                println!("{}", Uuid::new_v4());
            }
        }
        Commands::Base64 { decode, text } => {
            if decode {
                let bytes = STANDARD.decode(text)?;
                println!("{}", String::from_utf8_lossy(&bytes));
            } else {
                println!("{}", STANDARD.encode(text));
            }
        }
        Commands::Urlencode { decode, text } => {
            if decode {
                println!("{}", urlencoding::decode(&text)?);
            } else {
                println!("{}", urlencoding::encode(&text));
            }
        }
        Commands::HtmlEntities { decode, text } => {
            if decode {
                println!("{}", html_escape::decode_html_entities(&text));
            } else {
                println!("{}", html_escape::encode_text(&text));
            }
        }
        Commands::Hash { text } => {
            println!("MD5: {:x}", md5::compute(text.as_bytes()));
            println!("SHA256: {:x}", Sha256::digest(text.as_bytes()));
            println!("SHA512: {:x}", Sha512::digest(text.as_bytes()));
        }
        Commands::Jwt { token } => {
            let parts: Vec<&str> = token.split('.').collect();
            if parts.len() > 0 {
                if let Ok(h) = base64::engine::general_purpose::URL_SAFE_NO_PAD.decode(parts[0]) {
                    println!("Header:\n{}", String::from_utf8_lossy(&h));
                }
            }
            if parts.len() > 1 {
                if let Ok(p) = base64::engine::general_purpose::URL_SAFE_NO_PAD.decode(parts[1]) {
                    println!("Payload:\n{}", String::from_utf8_lossy(&p));
                }
            }
            if parts.len() > 2 {
                println!("Signature:\n{}", parts[2]);
            }
        }
        Commands::Stats { text } => {
            let chars = text.chars().count();
            let words = text.split_whitespace().count();
            let bytes = text.len();
            let lines = text.lines().count();
            println!("Chars: {}\nWords: {}\nLines: {}\nBytes: {}", chars, words, lines, bytes);
        }
        Commands::Ipv4 { cidr } => {
            let net = Ipv4Net::from_str(&cidr)?;
            println!("Network: {}", net.network());
            println!("Broadcast: {}", net.broadcast());
            println!("Netmask: {}", net.netmask());
            let hosts: Vec<_> = net.hosts().collect();
            println!("Total Hosts: {}", hosts.len());
            if hosts.len() > 0 {
                println!("Range: {} - {}", hosts.first().unwrap(), hosts.last().unwrap());
            }
        }
        Commands::Mac { count } => {
            let mut rng = rand::thread_rng();
            for _ in 0..count {
                let mut bytes = [0u8; 6];
                rng.fill(&mut bytes);
                bytes[0] = (bytes[0] | 0b0000_0010) & 0b1111_1110;
                println!("{:02X}:{:02X}:{:02X}:{:02X}:{:02X}:{:02X}",
                    bytes[0], bytes[1], bytes[2], bytes[3], bytes[4], bytes[5]);
            }
        }
        Commands::Yaml2json { text } => {
            let val: JsonValue = serde_yaml::from_str(&text)?;
            println!("{}", serde_json::to_string_pretty(&val)?);
        }
        Commands::Json2yaml { text } => {
            let val: JsonValue = serde_json::from_str(&text)?;
            println!("{}", serde_yaml::to_string(&val)?);
        }
        Commands::NumberBase { from, value } => {
            let parsed = match from.as_str() {
                "10" | "dec" => i64::from_str_radix(&value, 10),
                "16" | "hex" => i64::from_str_radix(&value.replace("0x", ""), 16),
                "2" | "bin" => i64::from_str_radix(&value.replace("0b", ""), 2),
                "8" | "oct" => i64::from_str_radix(&value.replace("0o", ""), 8),
                _ => return Err(anyhow::anyhow!("Invalid base. Use 10, 16, 2, or 8.")),
            }?;
            println!("Dec: {}", parsed);
            println!("Hex: {:X}", parsed);
            println!("Bin: {:b}", parsed);
            println!("Oct: {:o}", parsed);
        }
    }
    Ok(())
}
