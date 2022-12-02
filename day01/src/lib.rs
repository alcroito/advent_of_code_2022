use std::{path::Path, str::FromStr};

#[derive(Debug)]
struct Elf {
    snack_calories: Vec<u32>,
}

impl Elf {
    fn new(snack_calories: Vec<u32>) -> Self {
        Elf { snack_calories }
    }

    fn total_calories_carried(&self) -> u32 {
        self.snack_calories.iter().sum()
    }
}

impl FromStr for Elf {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let items = s
            .lines()
            .map(|l| l.trim())
            .filter(|s| !s.is_empty())
            .map(|s| {
                s.parse::<u32>()
                    .map_err(|e| Error::MalformedInput(e, s.to_owned()))
            })
            .collect::<Result<Vec<u32>, _>>()?;
        Ok(Elf::new(items))
    }
}

pub fn part1(input: &Path) -> Result<(), Error> {
    let res = std::fs::read_to_string(input)?
        .split("\n\n")
        .map(|e| e.parse::<Elf>())
        .collect::<Result<Vec<Elf>, Error>>()?
        .into_iter()
        .max_by_key(Elf::total_calories_carried)
        .ok_or(Error::NoSolution)?
        .total_calories_carried();
    println!("{}", res);
    Ok(())
}

pub fn part2(input: &Path) -> Result<(), Error> {
    let mut elfs = std::fs::read_to_string(input)?
        .split("\n\n")
        .map(|e| e.parse::<Elf>())
        .collect::<Result<Vec<Elf>, Error>>()?;
    elfs.sort_by_cached_key(Elf::total_calories_carried);
    let elfs = elfs;
    let res: u32 = elfs
        .iter()
        .rev()
        .take(3)
        .map(Elf::total_calories_carried)
        .sum();
    println!("{}", res);
    Ok(())
}

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error(transparent)]
    Io(#[from] std::io::Error),
    #[error("no solution found")]
    NoSolution,
    #[error("malformed input: {1}")]
    MalformedInput(#[source] std::num::ParseIntError, String),
}
