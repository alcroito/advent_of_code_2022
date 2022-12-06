use itertools::Itertools;
use std::path::Path;

type Stacks = Vec<Vec<char>>;
type Op = (usize, usize, usize);
type Ops = Vec<Op>;

fn parse_input(input: &Path) -> Result<(Stacks, Ops), Error> {
    let s = std::fs::read_to_string(input)?;
    let (stacks_str, ops_str) = s
        .split_once("\n\n")
        .ok_or(Error::StacksAndProcedureDelimiterNotFound)?;

    let stacks = parse_stacks(stacks_str)?;
    let ops = parse_ops(ops_str)?;
    Ok((stacks, ops))
}

fn parse_stacks(stacks_str: &str) -> Result<Stacks, Error> {
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

    Ok(stacks)
}

fn parse_ops(ops_str: &str) -> Result<Ops, Error> {
    let ops = ops_str
        .split('\n')
        .filter(|l| !l.is_empty())
        .map(|l| {
            let (_, count, _, from, _, to) =
                l.split(' ').collect_tuple().ok_or(Error::InvalidOp)?;
            Ok((count.parse()?, from.parse()?, to.parse()?))
        })
        .collect::<Result<Ops, Error>>()?;
    Ok(ops)
}

fn until_err<T, E>(err: &mut &mut Result<(), E>, item: Result<T, E>) -> Option<T> {
    match item {
        Ok(item) => Some(item),
        Err(e) => {
            **err = Err(e);
            None
        }
    }
}

fn get_two_mut<T>(slice: &mut [T], index1: usize, index2: usize) -> Option<(&mut T, &mut T)> {
    match index1.cmp(&index2) {
        std::cmp::Ordering::Greater => {
            let (start, end) = slice.split_at_mut(index1);
            Some((&mut end[0], &mut start[index2]))
        }
        std::cmp::Ordering::Less => {
            let (start, end) = slice.split_at_mut(index2);
            Some((&mut start[index1], &mut end[0]))
        }
        std::cmp::Ordering::Equal => None,
    }
}

fn run_ops(ops: Ops, stacks: &mut Stacks, p2: bool) -> Result<(), Error> {
    for (count, from, to) in ops {
        let mut err: Result<_, Error> = Ok(());

        let (stack_from, stack_to) =
            get_two_mut(stacks, from - 1, to - 1).ok_or(Error::InvalidOp)?;

        let extender = (0..count)
            .into_iter()
            .map(|_| {
                let item = stack_from.pop().ok_or(Error::StackEmpty(from))?;
                Ok(item)
            })
            .scan(&mut err, until_err);
        if p2 {
            stack_to.extend(extender.collect::<Vec<char>>().into_iter().rev());
        } else {
            stack_to.extend(extender);
        }
        err?;
    }
    Ok(())
}

fn get_stacks_top(stacks: &Stacks) -> Result<String, Error> {
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
    Ok(res)
}

pub fn part1(input: &Path) -> Result<(), Error> {
    let (mut stacks, ops) = parse_input(input)?;
    run_ops(ops, &mut stacks, false)?;
    println!("p1: {}", get_stacks_top(&stacks)?);
    Ok(())
}

pub fn part2(input: &Path) -> Result<(), Error> {
    let (mut stacks, ops) = parse_input(input)?;
    run_ops(ops, &mut stacks, true)?;
    println!("p2: {}", get_stacks_top(&stacks)?);
    Ok(())
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
