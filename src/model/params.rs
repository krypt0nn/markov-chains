use clap::Args;

#[derive(Debug, Clone, Copy, Args)]
pub struct GenerationParams {
    #[arg(long, default_value_t = 5)]
    /// Number of tokens used to generate the next token
    pub context_window: usize,

    #[arg(long, default_value_t = 0.15)]
    /// Probability to skip the most probable token
    /// 
    /// If `random_seed <= temperature * temperature_alpha^[token number] * repeat_penalty^[number of repeats]`,
    /// then the most probable token is skipped.
    pub temperature: f32,

    #[arg(long, default_value_t = 1.0)]
    /// Probability multiplier to skip the most probable token
    /// 
    /// See `temperature` for the formula.
    pub temperature_alpha: f32,

    #[arg(long, default_value_t = 1.1)]
    /// Probability multiplier for the temperature
    /// when the generated token was already generated before
    /// 
    /// See `temperature` for the formula.
    pub repeat_penalty: f32,

    #[arg(long, default_value_t = 1.0)]
    /// Multiplier of the random seed (from 0.0 to 1.0)
    /// which is used to determine if we should stop text generation
    pub end_weight: f32,

    #[arg(long, default_value_t = 0.85)]
    /// When we should stop generating the text
    /// 
    /// If `random_seed * end_weight >= end_height` and the current token
    /// is an ending, then we stop generating new tokens.
    pub end_height: f32,

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
            context_window: 3,
            temperature: 0.15,
            temperature_alpha: 1.0,
            repeat_penalty: 1.0,
            end_weight: 1.0,
            end_height: 0.7,
            min_length: 1,
            max_len: 25,
            force_break_len: 100
        }
    }
}
