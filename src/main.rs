mod core;
mod cli;

pub use core::*;

#[global_allocator]
static GLOBAL: mimalloc::MiMalloc = mimalloc::MiMalloc;

fn main() -> anyhow::Result<()> {
    cli::Cli::parse().execute()

    Ok(())
}
