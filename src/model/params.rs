use clap::Args;

#[derive(Debug, Clone, Copy, Args)]
pub struct GenerationParams {
    #[arg(long, default_value_t = 5)]
    /// Number of tokens used to generate the next token
    /// 
    /// If set to 0, then only the previous token is used.
    /// Affects performance the most.
    pub context_window: usize,

    #[arg(long, default_value_t = 0.15)]
    /// Probability to skip the most probable token
    /// 
    /// If `random_seed < temperature * temperature_alpha^[token number]`,
    /// then the most probable token is skipped.
    /// 
    /// `random_seed` is a random number from 0.0 to 1.0.
    pub temperature: f32,

    #[arg(long, default_value_t = 1.0)]
    /// Probability multiplier to skip the most probable token
    /// 
    /// See `temperature` for the formula.
    pub temperature_alpha: f32,

    #[arg(long, default_value_t = 0.8)]
    /// Probability multiplier for the temperature
    /// when the generated token was already generated before
    /// 
    /// If `random_seed < repeat_penalty^[repeats number]`,
    /// then the most probable token is skipped.
    /// 
    /// `random_seed` is a random number from 0.0 to 1.0.
    pub repeat_penalty: f32,

    #[arg(long, default_value_t = 0.85)]
    /// When we should stop generating the text
    /// 
    /// If `random_seed >= end_weight` and the current token
    /// is an ending, then we stop generating new tokens.
    /// 
    /// `random_seed` is a random number from 0.0 to 1.0.
    pub end_weight: f32,

    #[arg(long, default_value_t = 1)]
    /// Minimum length of the generated text
    pub min_length: usize,

    #[arg(long, default_value_t = 25)]
    /// Maximum length of the generated text
    /// 
    /// If the current token is an ending, and we have
    /// generated `max_len` tokens, then we stop generating new tokens.
    pub max_len: usize,

    #[arg(long, default_value_t = 100)]
    /// Absolute maximum length of the generated text
    /// 
    /// Breaks new tokens generation if we have generated
    /// `force_break_len` tokens.
    pub force_break_len: usize
}

impl Default for GenerationParams {
    #[inline]
    fn default() -> Self {
        Self {
            context_window: 5,
            temperature: 0.15,
            temperature_alpha: 1.0,
            repeat_penalty: 0.8,
            end_weight: 0.85,
            min_length: 1,
            max_len: 25,
            force_break_len: 100
        }
    }
}
