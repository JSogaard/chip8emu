use core::emulator::Emulator;

use anyhow::Result;
use clap::{Parser, Subcommand};
use disassembler::disassembler::disassembler;

const DEFAULT_SCALE: u32 = 20;

fn main() -> Result<()> {
    let cli = Cli::parse();
    match cli.command {
        Commands::Run {
            rom_path,
            window_scale,
        } => {
            let mut emulator = Emulator::new(&rom_path, window_scale)?;
            emulator.run()?;
        },
        Commands::Disassemble { rom_path, output } => {
            disassembler(&rom_path, output)?;
        }
    }

    Ok(())
}

#[derive(Parser)]
#[command(version)]
/// A CHIP-8 emulator/interpreter implemented in Rust.
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Run rom in emulator
    Run {
        rom_path: String,
        #[arg(short, long, default_value_t = DEFAULT_SCALE)]
        window_scale: u32,
    },
    /// Disassemble ROM
    Disassemble {
        rom_path: String,
        #[arg(short, long)]
        output: Option<String>,
    },
}
