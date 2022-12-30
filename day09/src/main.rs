use aoclib::{config::Config, website::get_input};
use day09::{part1, part2, run_gui};

// use color_eyre::eyre::Result;
use error_stack::{IntoReport, Result, ResultExt};
use std::path::PathBuf;
use structopt::StructOpt;

const YEAR: u32 = 2022;
const DAY: u8 = 9;

#[derive(StructOpt, Debug)]
struct RunArgs {
    /// input file
    #[structopt(long, parse(from_os_str))]
    input: Option<PathBuf>,

    /// skip part 1
    #[structopt(long)]
    no_part1: bool,

    /// run part 2
    #[structopt(long)]
    part2: bool,
}

impl RunArgs {
    fn input(&self) -> Result<PathBuf, AppError> {
        match self.input {
            None => {
                let config = Config::load().into_report().change_context(AppError)?;
                // this does nothing if the input file already exists, but
                // simplifies the workflow after cloning the repo on a new computer
                get_input(&config, YEAR, DAY)
                    .into_report()
                    .change_context(AppError)?;
                Ok(config.input_for(YEAR, DAY))
            }
            Some(ref path) => Ok(path.clone()),
        }
    }
}

fn main() -> Result<(), AppError> {
    // color_eyre::install().into_report()?;
    let args = RunArgs::from_args();
    let input_path = args.input()?;

    if !args.no_part1 {
        let res = part1(&input_path);
        res.change_context(AppError)?;
    }
    if args.part2 {
        let res = part2(&input_path);
        res.change_context(AppError)?;
    }

    let s = "R 4
U 4
L 3
D 1
R 4
D 1
L 5
R 2";
    // let s = std::fs::read_to_string(&input_path).unwrap();
    run_gui(s).change_context(AppError)?;
    Ok(())
}

#[derive(Debug, thiserror::Error)]
#[error("App failed")]
pub struct AppError;

// Disable switching to discrete GPU.
#[cfg(target_os = "macos")]
embed_plist::embed_info_plist!("Info.plist");
