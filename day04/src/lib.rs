use std::{ops::RangeInclusive, path::Path};

type Pair = (RangeInclusive<u8>, RangeInclusive<u8>);

fn parse_pair((i, s): (usize, &str)) -> Result<Pair, Error> {
    s.split_once(',')
        .ok_or(Error::InvalidPair(i))
        .and_then(|(l, r)| Ok((parse_range("left", l)?, parse_range("right", r)?)))
}

fn parse_range(side: &'static str, s: &str) -> Result<RangeInclusive<u8>, Error> {
    s.split_once('-')
        .ok_or(Error::InvalidRange(side))
        .and_then(|(lo, hi)| Ok(RangeInclusive::new(lo.parse::<u8>()?, hi.parse::<u8>()?)))
}

fn overlaps_fully((left, right): &Pair) -> bool {
    (left.contains(right.start()) && left.contains(right.end()))
        || (right.contains(left.start()) && right.contains(left.end()))
}

fn overlaps_partially((left, right): &Pair) -> bool {
    left.contains(right.start())
        || left.contains(right.end())
        || right.contains(left.start())
        || right.contains(left.end())
}

pub fn part1(input: &Path) -> Result<(), Error> {
    let res = std::fs::read_to_string(input)?
        .lines()
        .enumerate()
        .map(parse_pair)
        .collect::<Result<Vec<Pair>, Error>>()?
        .iter()
        .filter(|p| overlaps_fully(p))
        .count();
    println!("p1: {res}");
    Ok(())
}

pub fn part2(input: &Path) -> Result<(), Error> {
    let res = std::fs::read_to_string(input)?
        .lines()
        .enumerate()
        .map(parse_pair)
        .collect::<Result<Vec<Pair>, Error>>()?
        .iter()
        .filter(|p| overlaps_partially(p))
        .count();
    println!("p2: {res}");
    Ok(())
}

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error(transparent)]
    Io(#[from] std::io::Error),
    #[error(transparent)]
    InvalidBound(#[from] std::num::ParseIntError),
    #[error("Invalid range {0}")]
    InvalidRange(&'static str),
    #[error("Invalid pair {0}")]
    InvalidPair(usize),
}
