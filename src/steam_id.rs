#[derive(Debug, PartialEq)]
pub enum SteamInput {
    Steam64(u64),
    Vanity(String),
}

pub fn parse_input(raw: &str) -> Result<SteamInput, String> {
    let s = raw.trim();

    if s.is_empty() {
        return Err("input is empty".to_string());
    }

    if s.starts_with("http://") || s.starts_with("https://") {
        return parse_url(s);
    }

    if s.chars().all(|c| c.is_ascii_digit()) {
        return parse_steam64_str(s);
    }

    if s.chars()
        .all(|c| c.is_alphanumeric() || c == '-' || c == '_')
    {
        return Ok(SteamInput::Vanity(s.to_string()));
    }

    Err(format!("unrecognized input: {:?}", s))
}

fn parse_url(url: &str) -> Result<SteamInput, String> {
    let without_scheme = url
        .trim_start_matches("https://")
        .trim_start_matches("http://")
        .trim_end_matches('/');

    let parts: Vec<&str> = without_scheme.splitn(4, '/').collect();

    let host = parts.first().copied().unwrap_or("");
    if host != "steamcommunity.com" {
        return Err(format!("unrecognized host: {:?}", host));
    }

    let path_type = parts.get(1).copied().unwrap_or("");
    let value = parts.get(2).copied().unwrap_or("").trim_end_matches('/');

    match path_type {
        "profiles" => parse_steam64_str(value),
        "id" => {
            if value.is_empty() {
                Err("vanity handle in URL is empty".to_string())
            } else {
                Ok(SteamInput::Vanity(value.to_string()))
            }
        }
        other => Err(format!("unrecognized URL path: {:?}", other)),
    }
}

fn parse_steam64_str(s: &str) -> Result<SteamInput, String> {
    const LEN: usize = 17;
    const PREFIX: &str = "7656119";

    if s.len() != LEN {
        return Err(format!(
            "Steam64 ID must be {} digits, got {}",
            LEN,
            s.len()
        ));
    }

    if !s.starts_with(PREFIX) {
        return Err(format!("Steam64 ID must start with {}", PREFIX));
    }

    s.parse::<u64>()
        .map(SteamInput::Steam64)
        .map_err(|e| format!("parse error: {}", e))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn steam64_valid_is_parsed() {
        assert_eq!(
            parse_input("76561197960287930"),
            Ok(SteamInput::Steam64(76561197960287930))
        );
    }

    #[test]
    fn steam64_too_short_is_rejected() {
        assert!(parse_input("7656119796028793").is_err());
    }

    #[test]
    fn steam64_too_long_is_rejected() {
        assert!(parse_input("765611979602879300").is_err());
    }

    #[test]
    fn steam64_wrong_prefix_rejected() {
        assert!(parse_input("12345678901234567").is_err());
    }

    #[test]
    fn vanity_simple_handle_is_parsed() {
        assert_eq!(
            parse_input("gaben"),
            Ok(SteamInput::Vanity("gaben".to_string()))
        );
    }

    #[test]
    fn vanity_empty_string_is_rejected() {
        assert!(parse_input("").is_err());
    }

    #[test]
    fn vanity_whitespace_only_is_rejected() {
        assert!(parse_input("   ").is_err());
    }

    #[test]
    fn url_profiles_extracts_steam64() {
        assert_eq!(
            parse_input("https://steamcommunity.com/profiles/76561197960287930"),
            Ok(SteamInput::Steam64(76561197960287930))
        );
    }

    #[test]
    fn url_id_extracts_vanity() {
        assert_eq!(
            parse_input("https://steamcommunity.com/id/gaben"),
            Ok(SteamInput::Vanity("gaben".to_string()))
        );
    }

    #[test]
    fn url_trailing_slash_normalised() {
        assert_eq!(
            parse_input("https://steamcommunity.com/id/gaben/"),
            Ok(SteamInput::Vanity("gaben".to_string()))
        );
    }

    #[test]
    fn url_http_scheme_accepted() {
        assert_eq!(
            parse_input("http://steamcommunity.com/id/gaben"),
            Ok(SteamInput::Vanity("gaben".to_string()))
        );
    }

    #[test]
    fn url_wrong_domain_is_rejected() {
        assert!(parse_input("https://example.com/id/gaben").is_err());
    }

    #[test]
    fn url_profiles_bad_id_is_rejected() {
        assert!(parse_input("https://steamcommunity.com/profiles/notanumber").is_err());
    }
}
