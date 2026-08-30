use clap::{CommandFactory, Parser, Subcommand};
use clap_complete::{generate, Shell};
use std::io::{self, Read, Write};

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

pub fn print_completions(shell: Shell) -> anyhow::Result<()> {
    let mut cmd = Cli::command();
    let mut buf = Vec::new();
    generate(shell, &mut cmd, "tit", &mut buf);
    io::stdout().write_all(&buf)?;
    Ok(())
}

fn read_stdin() -> anyhow::Result<String> {
    let mut buf = String::new();
    io::stdin().read_to_string(&mut buf)?;
    Ok(buf)
}

/// Read value from argument if present, otherwise from stdin.
fn arg_or_stdin(text: Option<String>) -> anyhow::Result<String> {
    match text {
        Some(v) => Ok(v),
        None => read_stdin(),
    }
}

#[derive(Subcommand)]
pub enum Commands {
    /// Generate shell completion scripts
    Completions {
        /// Target shell
        #[arg(value_enum)]
        shell: Shell,
    },
    /// Generate UUIDs
    Uuid {
        #[arg(short, long, default_value_t = 1)]
        count: usize,
    },
    /// Base64 Encode/Decode
    Base64 {
        #[arg(short, long)]
        decode: bool,
        /// Value to encode/decode. Omit to read from stdin.
        text: Option<String>,
    },
    /// URL Encode/Decode
    Urlencode {
        #[arg(short, long)]
        decode: bool,
        /// Value to encode/decode. Omit to read from stdin.
        text: Option<String>,
    },
    /// HTML Entities Encode/Decode
    HtmlEntities {
        #[arg(short, long)]
        decode: bool,
        /// Value to encode/decode. Omit to read from stdin.
        text: Option<String>,
    },
    /// Generate Hashes (MD5, SHA256, SHA512)
    Hash {
        /// Value to hash. Omit to read from stdin.
        text: Option<String>,
    },
    /// Decode JWT token (optionally verify HMAC signature)
    Jwt {
        token: Option<String>,
        #[arg(short, long)]
        secret: Option<String>,
    },
    /// Text Statistics
    Stats {
        /// Value to analyze. Omit to read from stdin.
        text: Option<String>,
    },
    /// Format or minify JSON
    Json {
        #[arg(short, long)]
        minify: bool,
        /// JSON to format. Omit to read from stdin.
        text: Option<String>,
    },
    /// Convert text between common casings
    Case {
        /// lowercase, uppercase, title, camel, snake, kebab
        #[arg(short, long, default_value = "lowercase")]
        mode: String,
        /// Text to convert. Omit to read from stdin.
        text: Option<String>,
    },
    /// Validate and explain a standard five-field cron expression
    Cron { expression: String },
    /// Parse a URL into its constituent parts
    UrlParser { url: String },
    /// Convert between date/time formats
    Datetime {
        value: Option<String>,
        /// IANA timezone name, e.g. America/New_York
        #[arg(short, long)]
        timezone: Option<String>,
    },
    /// Generate Lorem Ipsum placeholder text
    Lorem {
        #[arg(short, long, default_value_t = 3)]
        paragraphs: usize,
    },
    /// Test a Rust regular expression against text
    Regex {
        /// Regular expression pattern
        pattern: String,
        /// Text to test against. Omit to read from stdin.
        text: Option<String>,
    },
    /// Convert a six-digit HEX color to RGB and HSL
    Color { hex: String },
    /// Calculate IPv4 subnet details from CIDR notation
    Ipv4 { cidr: String },
    /// Generate locally administered unicast MAC addresses
    Mac {
        #[arg(short, long, default_value_t = 1)]
        count: usize,
    },
    /// Convert YAML to pretty JSON
    Yaml2json { text: Option<String> },
    /// Convert JSON to YAML
    Json2yaml { text: Option<String> },
    /// Convert an integer from binary, octal, decimal, or hexadecimal
    NumberBase {
        #[arg(short, long)]
        from: String,
        value: Option<String>,
    },
    /// Generate secure random passwords
    Password {
        #[arg(short, long, default_value_t = 1)]
        count: usize,
        #[arg(short, long, default_value_t = 16)]
        length: usize,
    },
}

pub fn handle_cli(cmd: Commands) -> anyhow::Result<()> {
    match cmd {
        Commands::Completions { shell } => {
            return print_completions(shell);
        }
        Commands::Uuid { count } => {
            for _ in 0..count {
                println!("{}", uuid::Uuid::new_v4());
            }
        }
        Commands::Base64 { decode, text } => {
            let text = arg_or_stdin(text)?;
            if decode {
                println!(
                    "{}",
                    crate::tools::base64_encoder::decode_base64(&text)
                        .map_err(anyhow::Error::msg)?
                );
            } else {
                println!("{}", crate::tools::base64_encoder::encode_base64(&text));
            }
        }
        Commands::Urlencode { decode, text } => {
            let text = arg_or_stdin(text)?;
            if decode {
                println!("{}", urlencoding::decode(&text)?);
            } else {
                println!("{}", urlencoding::encode(&text));
            }
        }
        Commands::HtmlEntities { decode, text } => {
            let text = arg_or_stdin(text)?;
            if decode {
                println!("{}", html_escape::decode_html_entities(&text));
            } else {
                println!("{}", html_escape::encode_text(&text));
            }
        }
        Commands::Hash { text } => {
            let text = arg_or_stdin(text)?;
            let (md5_out, sha256_out, sha512_out) = crate::tools::hash_generator::hash_text(&text);
            println!("MD5: {md5_out}");
            println!("SHA256: {sha256_out}");
            println!("SHA512: {sha512_out}");
        }
        Commands::Jwt { token, secret } => {
            let token = arg_or_stdin(token)?;
            let (header, payload, signature) = crate::tools::jwt_parser::decode_jwt(&token);
            if !header.is_empty() {
                println!("Header:\n{header}");
            }
            if !payload.is_empty() {
                println!("Payload:\n{payload}");
            }
            if !signature.is_empty() {
                println!("Signature:\n{signature}");
            }
            if let Some(secret) = secret {
                match crate::tools::jwt_parser::verify_signature(&token, &secret) {
                    Ok(true) => println!("Signature: valid"),
                    Ok(false) => println!("Signature: invalid"),
                    Err(e) => println!("Signature verification error: {e}"),
                }
            }
        }
        Commands::Stats { text } => {
            let text = arg_or_stdin(text)?;
            let (chars, chars_no_spaces, words, lines, bytes) =
                crate::tools::text_stats::analyze(&text);
            println!("Characters (total): {chars}");
            println!("Characters (no spaces): {chars_no_spaces}");
            println!("Words: {words}");
            println!("Lines: {lines}");
            println!("Bytes: {bytes}");
        }
        Commands::Json { minify, text } => {
            let text = arg_or_stdin(text)?;
            let pretty = !minify;
            println!(
                "{}",
                crate::tools::json_formatter::format_json(&text, pretty)
                    .map_err(anyhow::Error::msg)?
            );
        }
        Commands::Case { mode, text } => {
            let text = arg_or_stdin(text)?;
            let converted = crate::tools::text_case_converter::convert_case(&text, &mode)
                .map_err(anyhow::Error::msg)?;
            println!("{converted}");
        }
        Commands::Cron { expression } => {
            println!(
                "{}",
                crate::tools::cron_parser::parse_cron(&expression).map_err(anyhow::Error::msg)?
            );
        }
        Commands::UrlParser { url } => {
            let parts = crate::tools::url_parser::parse_url(&url).map_err(anyhow::Error::msg)?;
            println!("Scheme:   {}", parts.scheme);
            println!("Host:     {}", parts.host);
            println!("Port:     {}", parts.port);
            println!("Path:     {}", parts.path);
            println!("Query:    {}", parts.query);
            println!("Fragment: {}", parts.fragment);
        }
        Commands::Datetime { value, timezone } => {
            let value = arg_or_stdin(value)?;
            let conversions = crate::tools::datetime::convert_datetime(&value, timezone.as_deref())
                .map_err(anyhow::Error::msg)?;
            for (name, val) in conversions {
                println!("{name:22} {val}");
            }
        }
        Commands::Lorem { paragraphs } => {
            println!(
                "{}",
                crate::tools::lorem_ipsum::generate(paragraphs.clamp(1, 50))
            );
        }
        Commands::Regex { pattern, text } => {
            let text = arg_or_stdin(text)?;
            println!(
                "{}",
                crate::tools::regex_tester::test_regex(&pattern, &text)
                    .map_err(anyhow::Error::msg)?
            );
        }
        Commands::Color { hex } => {
            println!(
                "{}",
                crate::tools::color_converter::convert_color(&hex).map_err(anyhow::Error::msg)?
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
        Commands::Yaml2json { text } => {
            let text = arg_or_stdin(text)?;
            println!(
                "{}",
                crate::tools::json_yaml_converter::yaml_to_json(&text)
                    .map_err(anyhow::Error::msg)?
            )
        }
        Commands::Json2yaml { text } => {
            let text = arg_or_stdin(text)?;
            println!(
                "{}",
                crate::tools::json_yaml_converter::json_to_yaml(&text)
                    .map_err(anyhow::Error::msg)?
            )
        }
        Commands::NumberBase { from, value } => {
            let value = arg_or_stdin(value)?;
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
        Commands::Password { count, length } => {
            let mut rng = rand::thread_rng();
            let passwords = crate::tools::password_generator::generate_passwords(
                &mut rng,
                count.clamp(1, 50),
                length.clamp(4, 128),
            );
            for pwd in passwords {
                println!("{pwd}");
            }
        }
    }
    Ok(())
}
