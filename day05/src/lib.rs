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
    let mut lines = stacks_str.split('\n').rev();

    let ids_char_count = lines
        .next()
        .map(|e| e.len())
        .ok_or(Error::CantDetermineStackCount)?;
    let stack_count = ids_char_count / 4 + 1;

    let stacks: Stacks = vec![vec![]; stack_count];
    let stacks = lines.fold(stacks, |stacks, l| {
        l.as_bytes()
            .chunks(4)
            .enumerate()
            .filter_map(|(col, e)| {
                if e[0] == b'[' {
                    Some((col, e[1] as char))
                } else {
                    None
                }
            })
            .fold(stacks, |mut stacks, (col, item)| {
                stacks[col].push(item);
                stacks
            })
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

#[allow(unused)]
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
    ops.into_iter().try_for_each(|(count, from, to)| {
        let (stack_from, stack_to) =
            get_two_mut(stacks, from - 1, to - 1).ok_or(Error::InvalidOp)?;

        let extender = (0..count)
            .into_iter()
            .map(|_| {
                let item = stack_from.pop().ok_or(Error::StackEmpty(from))?;
                Ok(item)
            })
            .collect::<Result<Vec<char>, Error>>()?
            .into_iter();
        match p2 {
            true => stack_to.extend(extender.rev()),
            false => stack_to.extend(extender),
        }
        Ok(())
    })
}

fn get_stacks_top(stacks: &Stacks) -> Result<String, Error> {
    let res = stacks
        .iter()
        .enumerate()
        .map(|(i, stack)| {
            let item = stack.last().cloned().ok_or(Error::StackEmpty(i))?;
            Ok(item)
        })
        .collect::<Result<String, Error>>()?;
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
