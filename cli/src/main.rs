use core::emulator::Emulator;

use anyhow::Result;
use clap::Parser;

const DEFAULT_SCALE: u32 = 20;

fn main() -> Result<()> {
    let cli = Cli::parse();
    let rom_path = cli.rom_path;
    let window_scale = cli.window_scale;

    let mut emulator = Emulator::new(&rom_path, window_scale)?;
    emulator.run()?;

    Ok(())
}

#[derive(Parser)]
struct Cli {
    rom_path: String,
    #[arg(short, long, default_value_t = DEFAULT_SCALE)]
    window_scale: u32,
}