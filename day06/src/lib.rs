use itertools::Itertools;
use std::path::Path;

fn find_marker(s: &str, window_size: usize) -> Result<usize, Error> {
    s.trim()
        .as_bytes()
        .windows(window_size)
        .enumerate()
        .find_map(|(i, w)| {
            if w.iter().all_unique() {
                Some(i + window_size)
            } else {
                None
            }
        })
        .ok_or(Error::NoSolution)
}

pub fn part1(input: &Path) -> Result<(), Error> {
    let res = find_marker(&std::fs::read_to_string(input)?, 4)?;
    println!("p1: {}", res);
    Ok(())
}

pub fn part2(input: &Path) -> Result<(), Error> {
    let res = find_marker(&std::fs::read_to_string(input)?, 14)?;
    println!("p2: {}", res);
    Ok(())
}

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error(transparent)]
    Io(#[from] std::io::Error),
    #[error("no solution found")]
    NoSolution,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test() -> Result<(), Error> {
        let s = "bvwbjplbgvbhsrlpgdmjqwftvncz";
        assert_eq!(find_marker(s, 4)?, 5);
        assert_eq!(find_marker(s, 14)?, 23);

        let s = "zcfzfwzzqfrljwzlrfnpqdbhtmscgvjw";
        assert_eq!(find_marker(s, 4)?, 11);
        assert_eq!(find_marker(s, 14)?, 26);
        Ok(())
    }
}
