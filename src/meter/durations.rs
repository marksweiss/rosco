use std::str::FromStr;
use derive_builder::Builder;
use crate::common::constants;

// Floating point factors for standard note durations
pub(crate) static WHOLE: f32 = 1.0;
pub(crate) static HALF: f32 = 0.5;
pub(crate) static QUARTER: f32 = 0.25;
pub(crate) static EIGHTH: f32 = 0.125;
pub(crate) static SIXTEENTH: f32 = 0.0625;
pub(crate) static THIRTY_SECOND: f32 = 0.03125;
pub(crate) static SIXTY_FOURTH: f32 = 0.015625;

#[derive(Clone, Copy, Debug, Hash, PartialEq, Eq)]
pub(crate) enum DurationType {
    Whole,
    Half,
    Quarter,
    Eighth,
    Sixteenth,
    ThirtySecond,
    SixtyFourth,
}

impl DurationType {
    pub(crate) fn to_factor(&self) -> f32 {
        match self {
            DurationType::Whole => WHOLE,
            DurationType::Half => HALF,
            DurationType::Quarter => QUARTER,
            DurationType::Eighth => EIGHTH,
            DurationType::Sixteenth => SIXTEENTH,
            DurationType::ThirtySecond => THIRTY_SECOND,
            DurationType::SixtyFourth => SIXTY_FOURTH,
        }
    }
}

impl FromStr for DurationType {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "Whole" => Ok(DurationType::Whole),
            "Half" => Ok(DurationType::Half),
            "Quarter" => Ok(DurationType::Quarter),
            "Eighth" => Ok(DurationType::Eighth),
            "Sixteenth" => Ok(DurationType::Sixteenth),
            "ThirtySecond" => Ok(DurationType::ThirtySecond),
            "SixtyFourth" => Ok(DurationType::SixtyFourth),
            "1" => Ok(DurationType::Whole),
            "1/2" => Ok(DurationType::Half),
            "1/4" => Ok(DurationType::Quarter),
            "1/8" => Ok(DurationType::Eighth),
            "1/16" => Ok(DurationType::Sixteenth),
            "1/32" => Ok(DurationType::ThirtySecond),
            "1/64" => Ok(DurationType::SixtyFourth),
            _ => Err(format!("Unknown duration type: {}", s)),
        }
    }
}

#[derive(Builder, Clone, Copy, Debug)]
#[builder(build_fn(validate = "Self::validate"))]
pub(crate) struct Duration {
    pub(crate) duration_type: DurationType,
    #[builder(setter(into))]
    pub(crate) duration_ms: f32,
}

impl DurationBuilder {
    pub(crate) fn validate(&self) -> Result<Duration, String> {
        let duration_type = self.duration_type.ok_or("Duration type is required")?;
        let duration_ms = self.duration_ms.ok_or("Duration in milliseconds is required")?;

        if duration_ms <= 0.0 {
            return Err(String::from("Duration: duration_ms must be greater than 0.0"));
        }

        Ok(Duration {
            duration_type,
            duration_ms,
        })
    }
}

impl Duration {
    #[allow(dead_code)]
    pub(crate) fn new(duration_type: DurationType, duration_ms: f32) -> Self {
        Self { duration_type, duration_ms }
    }

    #[allow(dead_code)]
    pub(crate) fn duration_factor(&self) -> f32 {
        self.duration_type.to_factor()
    }
}

impl PartialEq for Duration {
    fn eq(&self, other: &Self) -> bool {
        self.duration_type == other.duration_type &&
        (self.duration_ms - other.duration_ms).abs() < constants::FLOAT_EPSILON 
    }
}

impl Eq for Duration {}

#[cfg(test)]
mod test_duration {
    use super::*;

    #[test]
    fn test_duration_builder_success() {
        let duration = DurationBuilder::default()
            .duration_type(DurationType::Quarter)
            .duration_ms(500.0)
            .build()
            .unwrap();

        assert_eq!(duration.duration_type, DurationType::Quarter);
        assert_eq!(duration.duration_ms, 500.0);
        assert_eq!(duration.duration_factor(), QUARTER);
    }

    #[test]
    fn test_duration_builder_validation_negative_duration() {
        let result = DurationBuilder::default()
            .duration_type(DurationType::Half)
            .duration_ms(-100.0)
            .build();

        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("Duration: duration_ms must be greater than 0.0"));
    }

    #[test]
    fn test_duration_builder_validation_zero_duration() {
        let result = DurationBuilder::default()
            .duration_type(DurationType::Eighth)
            .duration_ms(0.0)
            .build();

        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("Duration: duration_ms must be greater than 0.0"));
    }

    #[test]
    fn test_duration_builder_missing_duration_type() {
        let result = DurationBuilder::default()
            .duration_ms(250.0)
            .build();

        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("Duration type is required"));
    }

    #[test]
    fn test_duration_builder_missing_duration_ms() {
        let result = DurationBuilder::default()
            .duration_type(DurationType::Whole)
            .build();

        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("Duration in milliseconds is required"));
    }

    #[test]
    fn test_duration_type_to_factor() {
        assert_eq!(DurationType::Whole.to_factor(), WHOLE);
        assert_eq!(DurationType::Half.to_factor(), HALF);
        assert_eq!(DurationType::Quarter.to_factor(), QUARTER);
        assert_eq!(DurationType::Eighth.to_factor(), EIGHTH);
        assert_eq!(DurationType::Sixteenth.to_factor(), SIXTEENTH);
        assert_eq!(DurationType::ThirtySecond.to_factor(), THIRTY_SECOND);
        assert_eq!(DurationType::SixtyFourth.to_factor(), SIXTY_FOURTH);
    }

    #[test]
    fn test_duration_equality() {
        let duration1 = DurationBuilder::default()
            .duration_type(DurationType::Quarter)
            .duration_ms(500.0)
            .build()
            .unwrap();

        let duration2 = DurationBuilder::default()
            .duration_type(DurationType::Quarter)
            .duration_ms(500.0)
            .build()
            .unwrap();

        assert_eq!(duration1, duration2);
    }
}
