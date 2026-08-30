use super::{copy_to_clipboard, Action, Category, Tool, ToolMeta};
use base64::{
    engine::general_purpose::URL_SAFE, engine::general_purpose::URL_SAFE_NO_PAD, Engine as _,
};
use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use hmac::{Hmac, Mac};
use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Style},
    widgets::{Block, Borders, Paragraph},
    Frame,
};
use serde_json::Value;
use sha2::{Sha256, Sha384, Sha512};
use tui_textarea::{Input, TextArea};

pub struct JwtParser<'a> {
    input: TextArea<'a>,
    secret_input: TextArea<'a>,
    header_out: String,
    payload_out: String,
    signature_out: String,
    verify_result: String,
    focus_secret: bool,
}

fn decode_part(part: &str) -> String {
    if part.is_empty() {
        return String::new();
    }

    let decode_result = URL_SAFE_NO_PAD.decode(part).or_else(|_| {
        let mut p = part.to_string();
        while !p.len().is_multiple_of(4) {
            p.push('=');
        }
        URL_SAFE.decode(&p)
    });

    match decode_result {
        Ok(b) => {
            if let Ok(s) = String::from_utf8(b) {
                if let Ok(v) = serde_json::from_str::<Value>(&s) {
                    return serde_json::to_string_pretty(&v).unwrap_or(s);
                }
                return s;
            }
            "Invalid UTF-8".to_string()
        }
        Err(_) => "Invalid Base64Url sequence".to_string(),
    }
}

pub fn decode_jwt(token: &str) -> (String, String, String) {
    let text = token.replace(['\n', '\r', ' '], "");
    if text.is_empty() {
        return (String::new(), String::new(), String::new());
    }

    let parts: Vec<&str> = text.split('.').collect();

    let header_out = if !parts.is_empty() {
        decode_part(parts[0])
    } else {
        String::new()
    };
    let payload_out = if parts.len() > 1 {
        decode_part(parts[1])
    } else {
        String::new()
    };
    let signature_out = if parts.len() > 2 {
        parts[2].to_string()
    } else {
        String::new()
    };
    (header_out, payload_out, signature_out)
}

type HmacSha256 = Hmac<Sha256>;
type HmacSha384 = Hmac<Sha384>;
type HmacSha512 = Hmac<Sha512>;

fn algorithm_from_header(header_json: &str) -> Option<String> {
    serde_json::from_str::<Value>(header_json)
        .ok()
        .and_then(|v| v.get("alg")?.as_str().map(|s| s.to_ascii_uppercase()))
}

fn base64url_encode(bytes: &[u8]) -> String {
    URL_SAFE_NO_PAD.encode(bytes)
}

pub fn verify_signature(token: &str, secret: &str) -> Result<bool, String> {
    let text = token.replace(['\n', '\r', ' '], "");
    let parts: Vec<&str> = text.split('.').collect();
    if parts.len() != 3 {
        return Err("JWT must have three parts".into());
    }

    let header_decoded = decode_part(parts[0]);
    let alg = algorithm_from_header(&header_decoded).unwrap_or_else(|| "HS256".to_string());

    let signing_input = format!("{}.{}", parts[0], parts[1]);
    let expected = match alg.as_str() {
        "HS256" => {
            let mut mac = HmacSha256::new_from_slice(secret.as_bytes())
                .map_err(|e| format!("Invalid key: {e}"))?;
            mac.update(signing_input.as_bytes());
            base64url_encode(&mac.finalize().into_bytes())
        }
        "HS384" => {
            let mut mac = HmacSha384::new_from_slice(secret.as_bytes())
                .map_err(|e| format!("Invalid key: {e}"))?;
            mac.update(signing_input.as_bytes());
            base64url_encode(&mac.finalize().into_bytes())
        }
        "HS512" => {
            let mut mac = HmacSha512::new_from_slice(secret.as_bytes())
                .map_err(|e| format!("Invalid key: {e}"))?;
            mac.update(signing_input.as_bytes());
            base64url_encode(&mac.finalize().into_bytes())
        }
        "NONE" => return Ok(parts[2].is_empty()),
        _ => return Err(format!("Unsupported algorithm: {alg}")),
    };

    Ok(parts[2] == expected)
}

impl<'a> JwtParser<'a> {
    pub fn new() -> Self {
        let mut input = TextArea::default();
        input.set_block(
            Block::default()
                .borders(Borders::ALL)
                .title(" Input JWT (Type here) "),
        );
        let mut secret_input = TextArea::default();
        secret_input.set_block(
            Block::default()
                .borders(Borders::ALL)
                .title(" Optional HMAC secret (Tab to focus) "),
        );
        Self {
            input,
            secret_input,
            header_out: String::new(),
            payload_out: String::new(),
            signature_out: String::new(),
            verify_result: String::new(),
            focus_secret: false,
        }
    }

    fn process(&mut self) {
        let token = self.input.lines().join("");
        (self.header_out, self.payload_out, self.signature_out) = decode_jwt(&token);
        let secret = self.secret_input.lines().join("");
        self.verify_result = if secret.is_empty() {
            String::new()
        } else {
            match verify_signature(&token, &secret) {
                Ok(true) => "Signature valid".into(),
                Ok(false) => "Signature invalid".into(),
                Err(e) => format!("Verify error: {e}"),
            }
        };
    }
}

impl<'a> Tool for JwtParser<'a> {
    fn meta(&self) -> ToolMeta {
        ToolMeta {
            id: "jwt-parser",
            name: "JWT Parser",
            category: Category::Development,
            description: "Decode JWT contents without verifying signatures.",
            keywords: &["jwt", "decode", "parser", "token", "json web token"],
        }
    }

    fn render(&mut self, f: &mut Frame, area: Rect, focused: bool) {
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints(
                [
                    Constraint::Length(4),      // Token input
                    Constraint::Length(3),      // Secret input
                    Constraint::Percentage(25), // Header
                    Constraint::Percentage(30), // Payload
                    Constraint::Length(3),      // Signature / verify status
                ]
                .as_ref(),
            )
            .split(area);

        let border_style = if focused {
            Style::default().fg(Color::Yellow)
        } else {
            Style::default()
        };

        if focused && !self.focus_secret {
            self.input.set_block(
                Block::default()
                    .borders(Borders::ALL)
                    .title(" Input Token (Esc to go back) ")
                    .border_style(border_style),
            );
            self.input.set_cursor_line_style(
                Style::default().add_modifier(ratatui::style::Modifier::UNDERLINED),
            );
        } else {
            self.input.set_block(
                Block::default()
                    .borders(Borders::ALL)
                    .title(" Input Token (Tab to secret) "),
            );
            self.input.set_cursor_line_style(Style::default());
        }

        if focused && self.focus_secret {
            self.secret_input.set_block(
                Block::default()
                    .borders(Borders::ALL)
                    .title(" HMAC secret ")
                    .border_style(border_style),
            );
            self.secret_input.set_cursor_line_style(
                Style::default().add_modifier(ratatui::style::Modifier::UNDERLINED),
            );
        } else {
            self.secret_input.set_block(
                Block::default()
                    .borders(Borders::ALL)
                    .title(" Optional HMAC secret (Tab) "),
            );
            self.secret_input.set_cursor_line_style(Style::default());
        }

        f.render_widget(&self.input, chunks[0]);
        f.render_widget(&self.secret_input, chunks[1]);

        let p_header = Paragraph::new(self.header_out.as_str()).block(
            Block::default()
                .borders(Borders::ALL)
                .title(" Header (Algorithm & Token Type) "),
        );
        f.render_widget(p_header, chunks[2]);

        let p_payload = Paragraph::new(self.payload_out.as_str()).block(
            Block::default()
                .borders(Borders::ALL)
                .title(" Payload (Data) "),
        );
        f.render_widget(p_payload, chunks[3]);

        let status = if self.verify_result.is_empty() {
            self.signature_out.clone()
        } else {
            format!("{} | {}", self.signature_out, self.verify_result)
        };
        let p_signature = Paragraph::new(status)
            .block(Block::default().borders(Borders::ALL).title(" Signature "));
        f.render_widget(p_signature, chunks[4]);
    }

    fn handle_key(&mut self, key: KeyEvent) -> Action {
        if key.code == KeyCode::Esc {
            return Action::Back;
        }

        if key.code == KeyCode::Tab {
            self.focus_secret = !self.focus_secret;
            return Action::None;
        }

        if matches!(key.code, KeyCode::Char('c') | KeyCode::Char('C'))
            && key.modifiers.contains(KeyModifiers::CONTROL)
        {
            return copy_to_clipboard(format!(
                "Header:\n{}\n\nPayload:\n{}\n\nSignature:\n{}\n\nVerification:\n{}",
                self.header_out,
                self.payload_out,
                self.signature_out,
                if self.verify_result.is_empty() {
                    "none"
                } else {
                    &self.verify_result
                }
            ));
        }

        let changed = if self.focus_secret {
            self.secret_input.input(Input::from(key))
        } else {
            self.input.input(Input::from(key))
        };
        if changed {
            self.process();
        }

        Action::None
    }

    fn help(&self) -> Vec<&'static str> {
        vec![
            "Decode only; optional HMAC verification",
            "Tab: token/secret pane",
            "Ctrl+C: copy decoded token",
        ]
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_hs256_token(secret: &str) -> String {
        let header = URL_SAFE_NO_PAD.encode(r#"{"alg":"HS256","typ":"JWT"}"#);
        let payload = URL_SAFE_NO_PAD.encode(r#"{"sub":"123"}"#);
        let mut mac = HmacSha256::new_from_slice(secret.as_bytes()).unwrap();
        mac.update(format!("{header}.{payload}").as_bytes());
        let sig = base64url_encode(&mac.finalize().into_bytes());
        format!("{header}.{payload}.{sig}")
    }

    #[test]
    fn decodes_json_payload() {
        assert!(decode_part("eyJzdWIiOiIxMjMifQ").contains("\"sub\": \"123\""));
    }

    #[test]
    fn rejects_invalid_base64url() {
        assert_eq!(decode_part("!!!"), "Invalid Base64Url sequence");
    }

    #[test]
    fn verifies_valid_hs256_signature() {
        let token = make_hs256_token("secret");
        assert!(verify_signature(&token, "secret").unwrap());
    }

    #[test]
    fn rejects_wrong_secret() {
        let token = make_hs256_token("secret");
        assert!(!verify_signature(&token, "wrong").unwrap());
    }
}
