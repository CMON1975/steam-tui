use thiserror::Error;

#[derive(Debug, PartialEq, Eq, Error)]
pub enum SteamIdError {
    #[error("input is empty")]
    Empty,

    #[error("Steam64 ID must be {expected} digits, got {actual}")]
    InvalidSteam64Length { expected: usize, actual: usize },

    #[error("Steam64 ID must start with {expected}")]
    InvalidSteam64Prefix { expected: &'static str },

    #[error("Steam64 ID contains non-digit characters")]
    InvalidSteam64Digits,

    #[error("unrecognized host: {0:?}")]
    UnrecognizedHost(String),

    #[error("unrecognized URL path: {0:?}")]
    UnrecognizedUrlPath(String),

    #[error("vanity handle in URL is empty")]
    EmptyVanityInUrl,

    #[error("unrecognized input: {0:?}")]
    Unrecognized(String),
}

#[derive(Debug, PartialEq)]
pub enum SteamInput {
    Steam64(u64),
    Vanity(String),
}

pub fn parse_input(raw: &str) -> Result<SteamInput, SteamIdError> {
    let s = raw.trim();

    if s.is_empty() {
        return Err(SteamIdError::Empty);
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

    Err(SteamIdError::Unrecognized(s.to_string()))
}

fn parse_url(url: &str) -> Result<SteamInput, SteamIdError> {
    let without_scheme = url
        .trim_start_matches("https://")
        .trim_start_matches("http://")
        .trim_end_matches('/');

    let parts: Vec<&str> = without_scheme.splitn(4, '/').collect();

    let host = parts.first().copied().unwrap_or("");
    if host != "steamcommunity.com" {
        return Err(SteamIdError::UnrecognizedHost(host.to_string()));
    }

    let path_type = parts.get(1).copied().unwrap_or("");
    let value = parts.get(2).copied().unwrap_or("").trim_end_matches('/');

    match path_type {
        "profiles" => parse_steam64_str(value),
        "id" => {
            if value.is_empty() {
                Err(SteamIdError::EmptyVanityInUrl)
            } else {
                Ok(SteamInput::Vanity(value.to_string()))
            }
        }
        other => Err(SteamIdError::UnrecognizedUrlPath(other.to_string())),
    }
}

fn parse_steam64_str(s: &str) -> Result<SteamInput, SteamIdError> {
    const LEN: usize = 17;
    const PREFIX: &str = "7656119";

    if s.len() != LEN {
        return Err(SteamIdError::InvalidSteam64Length {
            expected: LEN,
            actual: s.len(),
        });
    }

    if !s.starts_with(PREFIX) {
        return Err(SteamIdError::InvalidSteam64Prefix { expected: PREFIX });
    }

    s.parse::<u64>()
        .map(SteamInput::Steam64)
        .map_err(|_| SteamIdError::InvalidSteam64Digits)
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
        assert_eq!(
            parse_input("7656119796028793"),
            Err(SteamIdError::InvalidSteam64Length {
                expected: 17,
                actual: 16
            })
        );
    }

    #[test]
    fn steam64_too_long_is_rejected() {
        assert_eq!(
            parse_input("765611979602879300"),
            Err(SteamIdError::InvalidSteam64Length {
                expected: 17,
                actual: 18
            })
        );
    }

    #[test]
    fn steam64_wrong_prefix_rejected() {
        assert_eq!(
            parse_input("12345678901234567"),
            Err(SteamIdError::InvalidSteam64Prefix {
                expected: "7656119"
            })
        );
    }

    #[test]
    fn vanity_simple_handle_is_parsed() {
        assert_eq!(
            parse_input("gaben"),
            Ok(SteamInput::Vanity("gaben".to_string()))
        );
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
        assert_eq!(
            parse_input("https://example.com/id/gaben"),
            Err(SteamIdError::UnrecognizedHost("example.com".to_string()))
        );
    }

    #[test]
    fn url_profiles_bad_id_is_rejected() {
        assert_eq!(
            parse_input("https://steamcommunity.com/profiles/notanumber"),
            Err(SteamIdError::InvalidSteam64Length {
                expected: 17,
                actual: 10
            })
        );
    }

    #[test]
    fn url_unknown_path_type_is_rejected() {
        assert_eq!(
            parse_input("https://steamcommunity.com/groups/foo"),
            Err(SteamIdError::UnrecognizedUrlPath("groups".to_string()))
        );
    }

    #[test]
    fn steam64_correct_shape_but_non_digits_rejected() {
        // 17 chars, starts with 7656119, but the tail isn't all digits.
        // Reachable via the /profiles/ URL branch, which doesn't pre-check digits.
        assert_eq!(
            parse_input("https://steamcommunity.com/profiles/7656119abcdefghij"),
            Err(SteamIdError::InvalidSteam64Digits)
        );
    }

    #[test]
    fn mixed_garbage_is_unrecognized() {
        assert!(matches!(
            parse_input("foo@bar!"),
            Err(SteamIdError::Unrecognized(_))
        ));
    }
}
