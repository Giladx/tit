use base64::{engine::general_purpose::STANDARD, Engine as _};
use clap::{Parser, Subcommand};
use sha2::{Digest, Sha256, Sha512};
use uuid::Uuid;

#[derive(Parser)]
#[command(
    name = "tit",
    version,
    about = "Terminal UI toolbox and headless agent tools"
)]
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
    Hash { text: String },
    /// Decode JWT token
    Jwt { token: String },
    /// Text Statistics
    Stats { text: String },
    /// Calculate IPv4 subnet details from CIDR notation
    Ipv4 { cidr: String },
    /// Generate locally administered unicast MAC addresses
    Mac {
        #[arg(short, long, default_value_t = 1)]
        count: usize,
    },
    /// Convert YAML to pretty JSON
    Yaml2json { text: String },
    /// Convert JSON to YAML
    Json2yaml { text: String },
    /// Convert an integer from binary, octal, decimal, or hexadecimal
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
            if !parts.is_empty() {
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
            println!(
                "Chars: {}\nWords: {}\nLines: {}\nBytes: {}",
                chars, words, lines, bytes
            );
        }
        Commands::Ipv4 { cidr } => {
            let details =
                crate::tools::ipv4_subnet::calculate_subnet(&cidr).map_err(anyhow::Error::msg)?;
            println!("Network: {}", details.network);
            println!("Broadcast: {}", details.broadcast);
            println!("Netmask: {}", details.netmask);
            println!("Usable hosts: {}", details.usable_hosts);
            println!("Host range: {}", details.host_range);
        }
        Commands::Mac { count } => {
            let mut rng = rand::thread_rng();
            for _ in 0..count {
                println!("{}", crate::tools::mac_generator::generate_mac(&mut rng));
            }
        }
        Commands::Yaml2json { text } => println!(
            "{}",
            crate::tools::json_yaml_converter::yaml_to_json(&text).map_err(anyhow::Error::msg)?
        ),
        Commands::Json2yaml { text } => println!(
            "{}",
            crate::tools::json_yaml_converter::json_to_yaml(&text).map_err(anyhow::Error::msg)?
        ),
        Commands::NumberBase { from, value } => {
            let base = match from.to_ascii_lowercase().as_str() {
                "2" | "bin" | "binary" => 2,
                "8" | "oct" | "octal" => 8,
                "10" | "dec" | "decimal" => 10,
                "16" | "hex" | "hexadecimal" => 16,
                _ => return Err(anyhow::anyhow!("Invalid base; use 2, 8, 10, or 16")),
            };
            println!(
                "{}",
                crate::tools::number_base_converter::convert_number(&value, base)
                    .map_err(anyhow::Error::msg)?
            );
        }
    }
    Ok(())
}
