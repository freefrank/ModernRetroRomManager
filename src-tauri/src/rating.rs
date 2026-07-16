/// Convert ratings written by older MRRM versions and third-party metadata tools
/// to the 0.0-1.0 range used by EmulationStation internally.
///
/// Older MRRM versions could persist ScreenScraper's 0-20 score directly and
/// multiply it by 100 again on every Pegasus/EmulationStation conversion. Values
/// above 100 are therefore reduced first, then interpreted as either a 20-point
/// score or a percentage.
pub(crate) fn normalize_rating(mut value: f64) -> Option<f64> {
    if !value.is_finite() || value < 0.0 {
        return None;
    }

    while value > 100.0 {
        value /= 100.0;
    }

    let normalized = if value <= 1.0 {
        value
    } else if value <= 20.0 {
        value / 20.0
    } else {
        value / 100.0
    };

    Some(normalized.clamp(0.0, 1.0))
}

pub(crate) fn parse_rating(value: &str) -> Option<f64> {
    let value = value.trim().trim_end_matches('%').trim();
    if value.is_empty() {
        return None;
    }
    normalize_rating(value.parse::<f64>().ok()?)
}

pub(crate) fn deserialize_optional_f64<'de, D>(deserializer: D) -> Result<Option<f64>, D::Error>
where
    D: serde::Deserializer<'de>,
{
    let value = String::deserialize(deserializer)?;
    Ok(parse_rating(&value))
}

pub(crate) fn deserialize_optional_f32<'de, D>(deserializer: D) -> Result<Option<f32>, D::Error>
where
    D: serde::Deserializer<'de>,
{
    deserialize_optional_f64(deserializer).map(|rating| rating.map(|value| value as f32))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn normalizes_current_and_legacy_rating_scales() {
        assert_eq!(normalize_rating(0.8), Some(0.8));
        assert_eq!(normalize_rating(16.0), Some(0.8));
        assert_eq!(normalize_rating(80.0), Some(0.8));
        assert_eq!(normalize_rating(1600.0), Some(0.8));
        assert_eq!(normalize_rating(800_000.0), Some(0.8));
        assert_eq!(parse_rating("160000%"), Some(0.8));
        assert_eq!(parse_rating(""), None);
        assert_eq!(parse_rating("unknown"), None);
    }
}
use serde::Deserialize;
