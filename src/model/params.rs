use clap::Args;

#[derive(Debug, Clone, Copy, Args)]
pub struct GenerationParams {
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

    #[arg(long, default_value_t = 0.35)]
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

    #[arg(long, default_value_t = 1)]
    /// Minimum length of the generated text
    pub min_length: usize,

    #[arg(long, default_value_t = 150)]
    /// Maximum length of the generated text
    /// 
    /// Breaks new tokens generation if we have generated
    /// `max_len` tokens.
    pub max_len: usize,

    #[arg(long, default_value_t = false)]
    /// Do not use bigrams for text generation
    pub no_bigrams: bool,

    #[arg(long, default_value_t = false)]
    /// Do not use trigrams for text generation
    pub no_trigrams: bool
}

impl Default for GenerationParams {
    #[inline]
    fn default() -> Self {
        Self {
            temperature: 0.85,
            temperature_alpha: 1.0,
            repeat_penalty: 0.35,
            k_normal: 0.95,
            min_length: 1,
            max_len: 150,
            no_bigrams: false,
            no_trigrams: false
        }
    }
}
