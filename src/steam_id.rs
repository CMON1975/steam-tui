use thiserror::Error;

#[derive(Debug, PartialEq, Eq, Error)]
pub enum SteamIdError {
    #[error("input is empty")]
    Empty,

    #[error("Steam64 ID must be {expected} digits, got {actual}")]
    InvalidSteam64Length { expected: usize, actual: usize },

    #[error("vanity handle must be {min}-{max} characters, got {actual}")]
    InvalidVanityLength {
        min: usize,
        max: usize,
        actual: usize,
    },

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

    // Dispatch by length, not by character set.
    // Exactly 17 digits -> almost certainly an attempted Steam64 ID; let the
    // Steam64 validator produce a specific error (wrong prefix, etc.).
    // Anything else is a vanity handle candidate.
    if s.len() == 17 && s.chars().all(|c| c.is_ascii_digit()) {
        return parse_steam64_str(s);
    }

    parse_vanity(s)
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

fn parse_vanity(s: &str) -> Result<SteamInput, SteamIdError> {
    const MIN: usize = 3;
    const MAX: usize = 32;

    let len = s.chars().count();
    if !(MIN..=MAX).contains(&len) {
        return Err(SteamIdError::InvalidVanityLength {
            min: MIN,
            max: MAX,
            actual: len,
        });
    }

    if !s
        .chars()
        .all(|c| c.is_alphanumeric() || c == '-' || c == '_')
    {
        return Err(SteamIdError::Unrecognized(s.to_string()));
    }

    Ok(SteamInput::Vanity(s.to_string()))
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
    fn sixteen_digit_input_treated_as_vanity() {
        // With length-based dispatch, all-digit inputs of length != 17 are
        // vanity handles, not malformed Steam64 attempts. If the user really
        // meant a Steam64, the downstream API lookup will surface that.
        assert_eq!(
            parse_input("7656119796028793"),
            Ok(SteamInput::Vanity("7656119796028793".to_string()))
        );
    }

    #[test]
    fn eighteen_digit_input_treated_as_vanity() {
        // Same reasoning, on the other side of the length boundary.
        assert_eq!(
            parse_input("765611979602879300"),
            Ok(SteamInput::Vanity("765611979602879300".to_string()))
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

    #[test]
    fn numeric_vanity_handle_is_parsed() {
        // All-digit handles shorter than 17 chars should be treated as vanity,
        // not as malformed Steam64 IDs.
        assert_eq!(
            parse_input("12345"),
            Ok(SteamInput::Vanity("12345".to_string()))
        );
    }

    #[test]
    fn numeric_vanity_handle_at_max_length_is_parsed() {
        // 32 digits: valid vanity length, not 17, so not a Steam64 attempt.
        let handle = "1".repeat(32);
        assert_eq!(parse_input(&handle), Ok(SteamInput::Vanity(handle.clone())));
    }

    #[test]
    fn vanity_too_short_is_rejected() {
        assert_eq!(
            parse_input("ab"),
            Err(SteamIdError::InvalidVanityLength {
                min: 3,
                max: 32,
                actual: 2
            })
        );
    }

    #[test]
    fn vanity_too_long_is_rejected() {
        let handle = "a".repeat(33);
        assert_eq!(
            parse_input(&handle),
            Err(SteamIdError::InvalidVanityLength {
                min: 3,
                max: 32,
                actual: 33
            })
        );
    }

    #[test]
    fn seventeen_digit_non_steam64_still_tried_as_steam64() {
        // 17 digits but wrong prefix: user almost certainly meant a Steam64,
        // so the prefix error is more useful than a vanity error.
        assert_eq!(
            parse_input("12345678901234567"),
            Err(SteamIdError::InvalidSteam64Prefix {
                expected: "7656119"
            })
        );
    }
}
