use std::path::Path;

const ROCK: u32 = 1;
const PAPER: u32 = 2;
const SCISSORS: u32 = 3;

const WIN: u32 = 6;
const DRAW: u32 = 3;
const LOSS: u32 = 0;

/*
A X - Rock 1p
B Y - Paper 2p
C Z - Scissors 3p
*/
fn round_score_p1(s: &str) -> Result<u32, Error> {
    match s {
        "A X" => Ok(ROCK + DRAW),
        "A Y" => Ok(PAPER + WIN),
        "A Z" => Ok(SCISSORS + LOSS),
        "B X" => Ok(ROCK + LOSS),
        "B Y" => Ok(PAPER + DRAW),
        "B Z" => Ok(SCISSORS + WIN),
        "C X" => Ok(ROCK + WIN),
        "C Y" => Ok(PAPER + LOSS),
        "C Z" => Ok(SCISSORS + DRAW),
        _ => Err(Error::InvalidRound(s.to_owned())),
    }
}
/*
A - Rock     X - Lose
B - Paper    Y - Draw
C - Scissors Z - Win
*/
fn round_score_p2(s: &str) -> Result<u32, Error> {
    match s {
        "A X" => Ok(LOSS + SCISSORS),
        "A Y" => Ok(DRAW + ROCK),
        "A Z" => Ok(WIN + PAPER),
        "B X" => Ok(LOSS + ROCK),
        "B Y" => Ok(DRAW + PAPER),
        "B Z" => Ok(WIN + SCISSORS),
        "C X" => Ok(LOSS + PAPER),
        "C Y" => Ok(DRAW + SCISSORS),
        "C Z" => Ok(WIN + ROCK),
        _ => Err(Error::InvalidRound(s.to_owned())),
    }
}

pub fn part1(input: &Path) -> Result<(), Error> {
    let res: u32 = std::fs::read_to_string(input)?
        .lines()
        .map(round_score_p1)
        .collect::<Result<Vec<u32>, Error>>()?
        .iter()
        .sum();
    println!("{}", res);
    Ok(())
}

pub fn part2(input: &Path) -> Result<(), Error> {
    let res: u32 = std::fs::read_to_string(input)?
        .lines()
        .filter_map(|l| round_score_p2(l).ok())
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
    #[error("Invalid round move: {0}")]
    InvalidRound(String),
}
