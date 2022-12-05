use itertools::Itertools;
use std::path::Path;

type Stacks = Vec<Vec<char>>;
type Op = (usize, usize, usize);
type Ops = Vec<Op>;

pub fn part1(input: &Path) -> Result<(), Error> {
    let s = std::fs::read_to_string(input)?;
    let (stacks_str, ops_str) = s
        .split_once("\n\n")
        .ok_or(Error::StacksAndProcedureDelimiterNotFound)?;

    let mut lines = stacks_str.split('\n').rev().peekable();

    let ids_char_count = lines
        .peek()
        .map(|e| e.len())
        .ok_or(Error::CantDetermineStackCount)?;
    let stack_count = (ids_char_count - 3) / 4 + 1;

    let mut stacks: Stacks = vec![vec![]; stack_count];

    lines.skip(1).for_each(|l| {
        l.as_bytes()
            .chunks(4)
            .map(|e| {
                if e[0] == b'[' {
                    Some(e[1] as char)
                } else {
                    None
                }
            })
            .enumerate()
            .for_each(|(i, e)| {
                if let Some(item) = e {
                    stacks[i].push(item)
                }
            });
    });

    let ops = ops_str
        .split('\n')
        .filter(|l| !l.is_empty())
        .map(|l| {
            let (_, count, _, from, _, to) =
                l.split(' ').collect_tuple().ok_or(Error::InvalidOp)?;
            Ok((count.parse()?, from.parse()?, to.parse()?))
        })
        .collect::<Result<Ops, Error>>()?;

    for (count, from, to) in ops {
        (0..count)
            .into_iter()
            .map(|_| {
                let item = stacks[from - 1].pop().ok_or(Error::StackEmpty(from))?;
                stacks[to - 1].push(item);
                Ok(())
            })
            .collect::<Result<Vec<()>, Error>>()?;
    }

    let res = stacks
        .iter()
        .enumerate()
        .map(|(i, stack)| {
            let item = stack.last().cloned().ok_or(Error::StackEmpty(i))?;
            Ok(item)
        })
        .collect::<Result<Vec<char>, Error>>()?
        .iter()
        .collect::<String>();

    println!("p1: {}", res);
    Ok(())
}

pub fn part2(input: &Path) -> Result<(), Error> {
    unimplemented!("input file: {:?}", input)
}

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error(transparent)]
    Io(#[from] std::io::Error),
    #[error(transparent)]
    InvalidOpNumber(#[from] std::num::ParseIntError),
    #[error("Could not find stacks and procedure delimiter not found")]
    StacksAndProcedureDelimiterNotFound,
    #[error("Could not detect stack count")]
    CantDetermineStackCount,
    #[error("Invalid operation")]
    InvalidOp,
    #[error("No more elements in stack {0}")]
    StackEmpty(usize),
}
