use clap::{Parser, Subcommand};
use uuid::Uuid;
use base64::{Engine as _, engine::general_purpose::STANDARD};
use sha2::{Sha256, Sha512, Digest};

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
                println!("{}", html_escape::encode_html_entity(&text));
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
    }
    Ok(())
}
