#[derive(Default, Debug, Clone, Copy, PartialEq, Eq, Hash)]
/// Supported compression algorithms.
///
/// - Source 1: https://manishrjain.com/compression-algo-moving-data
/// - Source 2: https://linuxreviews.org/Comparison_of_Compression_Algorithms
/// - Source 3: https://cran.r-project.org/web/packages/brotli/vignettes/brotli-2015-09-22.pdf
pub enum Compression {
    #[default]
    /// Very fast compression and decompression speed,
    /// small compression rate.
    Lz4,

    /// Fast compression and decompression speed,
    /// very good compression rate.
    Zstd,

    /// Very slow compression speed, good decompression speed,
    /// good compression rate for English text.
    Brotli
}

impl Compression {
    pub fn compress(&self, mut data: &[u8]) -> anyhow::Result<Vec<u8>> {
        match self {
            Self::Lz4 => Ok(lz4_flex::compress(data)),

            // Use default compression level.
            // Subject of discussions.
            Self::Zstd => Ok(zstd::encode_all(data, 0)?),

            Self::Brotli => {
                let mut compressed = Vec::new();

                let size_hint = data.len();

                brotli::BrotliCompress(&mut data, &mut compressed, &brotli::enc::BrotliEncoderParams {
                    mode: brotli::enc::backward_references::BrotliEncoderMode::BROTLI_MODE_TEXT,

                    // Reduce compression rate in favor of speed.
                    // Subject of discussions.
                    quality: 6,

                    size_hint,

                    ..Default::default()
                })?;

                Ok(compressed)
            }
        }
    }

    pub fn decompress(&self, mut data: &[u8]) -> anyhow::Result<Vec<u8>> {
        match self {
            Self::Lz4 => {
                let mut buf_size = data.len() * 2;

                loop {
                    match lz4_flex::decompress(data, buf_size) {
                        Ok(data) => return Ok(data),
                        Err(lz4_flex::block::DecompressError::OutputTooSmall { .. }) => buf_size *= 2,
                        Err(err) => anyhow::bail!(err)
                    }
                }
            }

            Self::Zstd => Ok(zstd::decode_all(data)?),

            Self::Brotli => {
                let mut decompressed = Vec::with_capacity(data.len());

                brotli::BrotliDecompress(&mut data, &mut decompressed)?;

                Ok(decompressed)
            }
        }
    }
}
