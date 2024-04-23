use clap::{
    Args,
    ValueEnum
};

use clap::builder::{
    PossibleValue,
    EnumValueParser
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SmoothingAlgorithm {
    /// Kneser–Ney smoothing - https://en.wikipedia.org/wiki/Kneser%E2%80%93Ney_smoothing
    KneserNay,

    /// Absolute discounting smoothing
    AbsoluteDiscounting
}

impl ValueEnum for SmoothingAlgorithm {
    #[inline]
    fn value_variants<'a>() -> &'a [Self] {
        &[Self::KneserNay, Self::AbsoluteDiscounting]
    }

    #[inline]
    fn to_possible_value(&self) -> Option<PossibleValue> {
        Some(match self {
            Self::KneserNay => PossibleValue::new("kneser-ney"),
            Self::AbsoluteDiscounting => PossibleValue::new("absolute-discounting"),
        })
    }
}

#[derive(Debug, Clone, Copy, Args)]
pub struct GenerationParams {
    #[arg(long, default_value_t = 15)]
    /// Number of tokens used to generate the next token
    /// 
    /// If set to 0, then only the previous token is used.
    /// Affects performance the most.
    pub context_window: usize,

    #[arg(long, default_value_t = 0.85)]
    /// Probability to keep the most probable token
    /// 
    /// If `random_seed > temperature * temperature_alpha^[token number]`,
    /// then the most probable token is skipped.
    /// 
    /// Lower temperature generates more random text.
    /// 
    /// `random_seed` is a random number from 0.0 to 1.0.
    pub temperature: f64,

    #[arg(long, default_value_t = 1.0)]
    /// Probability multiplier to skip the most probable token
    /// 
    /// See `temperature` for the formula.
    pub temperature_alpha: f64,

    #[arg(long, default_value_t = 0.4)]
    /// Probability to skip repeated token
    /// 
    /// If `random_seed > repeat_penalty^[repeats number]`,
    /// then the repeated token is skipped.
    /// 
    /// Lower penalty skips repeated tokens more aggressively.
    /// 
    /// `random_seed` is a random number from 0.0 to 1.0.
    pub repeat_penalty: f64,

    #[arg(long, default_value_t = 0.95)]
    /// Percent of tokens to keep from the normal distribution
    /// 
    /// Other tokens will be removed equally from the beginning
    /// (least probable) and end (most probable).
    /// 
    /// Lower value will generate more "bot-looking" (weird) text.
    pub k_normal: f64,

    #[arg(
        long,
        value_parser = EnumValueParser::<SmoothingAlgorithm>::new()
    )]
    pub smoothing: Option<SmoothingAlgorithm>,

    #[arg(long, default_value_t = 1)]
    /// Minimum length of the generated text
    pub min_length: usize,

    #[arg(long, default_value_t = 150)]
    /// Maximum length of the generated text
    /// 
    /// Breaks new tokens generation if we have generated
    /// `max_len` tokens.
    pub max_len: usize
}

impl Default for GenerationParams {
    #[inline]
    fn default() -> Self {
        Self {
            context_window: 15,
            temperature: 0.85,
            temperature_alpha: 1.0,
            repeat_penalty: 0.4,
            k_normal: 0.95,
            smoothing: None,
            min_length: 1,
            max_len: 150
        }
    }
}
