use std::{
    ops::{Index, IndexMut},
    path::Path,
};
use tailsome::IntoResult;

const VISIBLE: i8 = -1;
type Value = i8;
type Idx = (usize, usize);

#[derive(Debug, Clone)]
struct Grid {
    g: Vec<Value>,
    #[allow(unused)]
    rows: usize,
    cols: usize,
}

impl Grid {
    fn new(rows: usize, cols: usize) -> Self {
        Grid {
            g: vec![VISIBLE; rows * cols],
            rows,
            cols,
        }
    }
}

impl std::fmt::Display for Grid {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        for row in 0..self.rows {
            for col in 0..self.cols {
                write!(f, "{}", self[(row, col)])?;
            }
            writeln!(f)?;
        }
        Ok(())
    }
}

impl Index<Idx> for Grid {
    type Output = Value;

    fn index(&self, index: Idx) -> &Self::Output {
        let row = index.0;
        let col = index.1;
        &self.g[row * self.cols + col]
    }
}

impl IndexMut<Idx> for Grid {
    fn index_mut(&mut self, index: Idx) -> &mut Self::Output {
        let row = index.0;
        let col = index.1;
        &mut self.g[row * self.cols + col]
    }
}

enum RayDirection {
    LeftToRight,
    RightToLeft,
    TopToBottom,
    BottomToTop,
}

impl RayDirection {
    fn side_index(&self) -> usize {
        match self {
            RayDirection::LeftToRight => 0,
            RayDirection::RightToLeft => 1,
            RayDirection::TopToBottom => 2,
            RayDirection::BottomToTop => 3,
        }
    }
}

struct Forest {
    heightmap: Grid,
    cover_sides: Vec<Grid>,
}

impl Forest {
    fn new(heightmap: Grid) -> Self {
        let rows = heightmap.rows;
        let cols = heightmap.cols;
        Forest {
            heightmap,
            cover_sides: vec![Grid::new(rows, cols); 4],
        }
    }

    fn compute_visibility(&mut self) {
        let side = RayDirection::LeftToRight.side_index();
        for r in 0..self.heightmap.rows {
            let mut highest = VISIBLE;
            for c in 0..self.heightmap.cols {
                self.cover_sides[side][(r, c)] = highest;
                highest = std::cmp::max(highest, self.heightmap[(r, c)]);
            }
        }

        let side = RayDirection::RightToLeft.side_index();
        for r in 0..self.heightmap.rows {
            let mut highest = VISIBLE;
            for c in (0..self.heightmap.cols).rev() {
                self.cover_sides[side][(r, c)] = highest;
                highest = std::cmp::max(highest, self.heightmap[(r, c)]);
            }
        }

        let side = RayDirection::TopToBottom.side_index();
        for c in 0..self.heightmap.cols {
            let mut highest = VISIBLE;
            for r in 0..self.heightmap.rows {
                self.cover_sides[side][(r, c)] = highest;
                highest = std::cmp::max(highest, self.heightmap[(r, c)]);
            }
        }

        let side = RayDirection::BottomToTop.side_index();
        for c in 0..self.heightmap.cols {
            let mut highest = VISIBLE;
            for r in (0..self.heightmap.rows).rev() {
                self.cover_sides[side][(r, c)] = highest;
                highest = std::cmp::max(highest, self.heightmap[(r, c)]);
            }
        }
    }

    fn is_tree_visible(&self, row: usize, col: usize) -> bool {
        (0..self.cover_sides.len())
            .into_iter()
            .map(|side| self.heightmap[(row, col)] > self.cover_sides[side][(row, col)])
            .any(|visible| visible)
    }

    fn count_visible_trees(&self) -> usize {
        let mut count = 0;
        for r in 0..self.heightmap.rows {
            for c in 0..self.heightmap.cols {
                if self.is_tree_visible(r, c) {
                    count += 1;
                }
            }
        }
        count
    }

    fn get_tree_scenic_score(&self, row: usize, col: usize) -> usize {
        let mut scenic_score = 1;
        let tree_height = self.heightmap[(row, col)];

        // LeftToRight
        let mut visible = 0;
        for c in col + 1..self.heightmap.cols {
            visible += 1;
            if self.heightmap[(row, c)] >= tree_height {
                break;
            }
        }
        scenic_score *= visible;

        // Right To Left
        let mut visible = 0;
        for c in (0..col).rev() {
            visible += 1;
            if self.heightmap[(row, c)] >= tree_height {
                break;
            }
        }
        scenic_score *= visible;

        // TopToBottom
        let mut visible = 0;
        for r in row + 1..self.heightmap.rows {
            visible += 1;
            if self.heightmap[(r, col)] >= tree_height {
                break;
            }
        }
        scenic_score *= visible;

        // BottomToTop
        let mut visible = 0;
        for r in (0..row).rev() {
            visible += 1;
            if self.heightmap[(r, col)] >= tree_height {
                break;
            }
        }
        scenic_score *= visible;
        scenic_score
    }

    fn find_highest_scenic_score(&self) -> usize {
        let mut highest = 0;
        for r in 0..self.heightmap.rows {
            for c in 0..self.heightmap.cols {
                highest = std::cmp::max(highest, self.get_tree_scenic_score(r, c));
            }
        }
        highest
    }
}

fn parse_grid(s: &str) -> Result<Grid, Error> {
    let row_count = s.split('\n').count();

    s.split('\n')
        .enumerate()
        .flat_map(|(row, l)| {
            l.chars().enumerate().map(move |(col, c)| {
                c.to_digit(10)
                    .ok_or(Error::InvalidHeight(c))
                    .map(|height| (row, col, height as Value))
            })
        })
        .try_fold(Grid::new(row_count, row_count), |mut grid, t| {
            let (row, col, h) = t?;
            grid[(row, col)] = h;
            grid.into_ok()
        })
}

pub fn part1(input: &Path) -> Result<(), Error> {
    let s = std::fs::read_to_string(input)?;
    let grid = parse_grid(&s)?;
    let mut forest = Forest::new(grid);
    forest.compute_visibility();
    let res = forest.count_visible_trees();
    println!("p1: {}", res);
    Ok(())
}

pub fn part2(input: &Path) -> Result<(), Error> {
    let s = std::fs::read_to_string(input)?;
    let grid = parse_grid(&s)?;
    let forest = Forest::new(grid);
    let res = forest.find_highest_scenic_score();
    println!("p2: {}", res);
    Ok(())
}

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error(transparent)]
    Io(#[from] std::io::Error),
    #[error("no solution found")]
    NoSolution,
    #[error("Invalid height {0}")]
    InvalidHeight(char),
}
