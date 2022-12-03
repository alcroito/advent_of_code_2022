use itertools::Itertools;
use std::collections::HashSet;
use std::path::Path;

fn priority(c: char) -> u32 {
    if c.is_ascii_lowercase() {
        c as u32 - 'a' as u32 + 1
    } else if c.is_ascii_uppercase() {
        c as u32 - 'A' as u32 + 27
    } else {
        unreachable!()
    }
}

pub fn part1(input: &Path) -> Result<(), Error> {
    let res: u32 = std::fs::read_to_string(input)?
        .lines()
        .enumerate()
        .map(|(i, rucksack)| {
            let len = rucksack.len();
            assert!(len % 2 == 0);
            let middle = len / 2;
            let compartment_1 = &rucksack[0..middle].chars().collect::<HashSet<char>>();
            rucksack[middle..len]
                .chars()
                .find(|c| compartment_1.contains(c))
                .ok_or(Error::NoMisplacedItem(i))
                .map(priority)
        })
        .collect::<Result<Vec<u32>, Error>>()?
        .iter()
        .sum();
    println!("p1: {}", res);
    Ok(())
}

pub fn part2(input: &Path) -> Result<(), Error> {
    let res: u32 = std::fs::read_to_string(input)?
        .lines()
        .chunks(3)
        .into_iter()
        .enumerate()
        .map(|(i, group)| {
            group
                .map(|rucksack| rucksack.chars().collect::<HashSet<char>>())
                .reduce(|acc, rucksack| {
                    acc.intersection(&rucksack)
                        .map(|e| e.to_owned())
                        .collect::<HashSet<char>>()
                })
                .ok_or(Error::NoGroup(i))?
                .iter()
                .next()
                .ok_or(Error::NoBadgeFound(i))
                .map(|item| priority(*item))
        })
        .collect::<Result<Vec<u32>, Error>>()?
        .iter()
        .sum();
    println!("p2: {res}");
    Ok(())
}

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error(transparent)]
    Io(#[from] std::io::Error),
    #[error("no misplaced item in rucksack {0}")]
    NoMisplacedItem(usize),
    #[error("no badge found in group {0}")]
    NoBadgeFound(usize),
    #[error("Invalid group with no elves {0}")]
    NoGroup(usize),
}
