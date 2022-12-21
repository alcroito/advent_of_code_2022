use std::{
    ops::{Index, IndexMut},
    path::Path,
};
use tailsome::IntoResult;

const VISIBLE: i8 = -1;
type Value = i8;
type Idx = (usize, usize);

#[derive(Clone, Copy)]
enum GridIterDirection {
    RowsLeftToRight,
    RowsRightToLeft,
    ColsTopToBottom,
    ColsBottomToTop,
}

impl GridIterDirection {
    fn side_index(&self) -> usize {
        match self {
            GridIterDirection::RowsLeftToRight => 0,
            GridIterDirection::RowsRightToLeft => 1,
            GridIterDirection::ColsTopToBottom => 2,
            GridIterDirection::ColsBottomToTop => 3,
        }
    }
}

#[allow(unused)]
type BoxedAxisIter<'a> = Box<dyn Iterator<Item = &'a Value> + 'a>;
#[allow(unused)]
type BoxedGridIter<'a> = Box<dyn Iterator<Item = BoxedAxisIter<'a>> + 'a>;
type BoxedAxisIndexIter<'a> = Box<dyn Iterator<Item = Idx> + 'a>;
type BoxedGridIndexIter<'a> = Box<dyn Iterator<Item = BoxedAxisIndexIter<'a>> + 'a>;

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

    fn get_element_index(&self, row: usize, col: usize) -> usize {
        row * self.cols + col
    }

    fn row_index_iter(&self, row: usize) -> impl DoubleEndedIterator<Item = Idx> {
        let start = 0;
        let end = self.cols;
        (start..end).map(move |col| (row, col))
    }

    fn row_index_iter_rev(&self, row: usize) -> impl DoubleEndedIterator<Item = Idx> {
        self.row_index_iter(row).rev()
    }

    fn col_index_iter(&self, col: usize) -> impl DoubleEndedIterator<Item = Idx> {
        let start = 0;
        let end = self.rows;
        (start..end).map(move |row| (row, col))
    }

    fn col_index_iter_rev(&self, col: usize) -> impl DoubleEndedIterator<Item = Idx> {
        self.col_index_iter(col).rev()
    }

    #[allow(unused)]
    fn index_iter_dynamic_dispatch(&self, dir: &GridIterDirection) -> BoxedGridIndexIter {
        match dir {
            GridIterDirection::RowsLeftToRight => Box::new(
                (0..self.rows).map(|row| Box::new(self.row_index_iter(row)) as BoxedAxisIndexIter),
            ) as BoxedGridIndexIter,
            GridIterDirection::RowsRightToLeft => Box::new(
                (0..self.rows)
                    .map(|row| Box::new(self.row_index_iter_rev(row)) as BoxedAxisIndexIter),
            ) as BoxedGridIndexIter,
            GridIterDirection::ColsTopToBottom => Box::new(
                (0..self.cols).map(|col| Box::new(self.col_index_iter(col)) as BoxedAxisIndexIter),
            ) as BoxedGridIndexIter,
            GridIterDirection::ColsBottomToTop => Box::new(
                (0..self.cols)
                    .map(|col| Box::new(self.col_index_iter_rev(col)) as BoxedAxisIndexIter),
            ) as BoxedGridIndexIter,
        }
    }

    fn axis_index_iter_static(
        &self,
        axis_index: usize,
        dir: &GridIterDirection,
    ) -> GridAxisIterator<
        impl Iterator<Item = Idx>,
        impl Iterator<Item = Idx>,
        impl Iterator<Item = Idx>,
        impl Iterator<Item = Idx>,
    > {
        match dir {
            GridIterDirection::RowsLeftToRight => {
                GridAxisIterator::RowLeftToRight(self.row_index_iter(axis_index))
            }
            GridIterDirection::RowsRightToLeft => {
                GridAxisIterator::RowRightToLeft(self.row_index_iter_rev(axis_index))
            }
            GridIterDirection::ColsTopToBottom => {
                GridAxisIterator::ColTopToBottom(self.col_index_iter(axis_index))
            }
            GridIterDirection::ColsBottomToTop => {
                GridAxisIterator::ColBottomToTop(self.col_index_iter_rev(axis_index))
            }
        }
    }

    fn index_iter_static_dispatch(
        &self,
        dir: &GridIterDirection,
    ) -> impl Iterator<Item = impl Iterator<Item = Idx>> + '_ {
        let dir = *dir;
        match dir {
            GridIterDirection::RowsLeftToRight => GridIterator::RowsLeftToRight(
                (0..self.rows).map(move |row| self.axis_index_iter_static(row, &dir)),
            ),
            GridIterDirection::RowsRightToLeft => GridIterator::RowsRightToLeft(
                (0..self.rows).map(move |row| self.axis_index_iter_static(row, &dir)),
            ),
            GridIterDirection::ColsTopToBottom => GridIterator::ColsTopToBottom(
                (0..self.cols).map(move |col| self.axis_index_iter_static(col, &dir)),
            ),
            GridIterDirection::ColsBottomToTop => GridIterator::ColsBottomToTop(
                (0..self.cols).map(move |col| self.axis_index_iter_static(col, &dir)),
            ),
        }
    }
}

impl std::fmt::Display for Grid {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        (0..self.rows).try_for_each(|row| {
            (0..self.cols).try_for_each(|col| write!(f, "{}", self[(row, col)]))?;
            writeln!(f)
        })
    }
}

impl Index<Idx> for Grid {
    type Output = Value;

    fn index(&self, index: Idx) -> &Self::Output {
        let row = index.0;
        let col = index.1;
        let index = self.get_element_index(row, col);
        &self.g[index]
    }
}

impl IndexMut<Idx> for Grid {
    fn index_mut(&mut self, index: Idx) -> &mut Self::Output {
        let row = index.0;
        let col = index.1;
        let index = self.get_element_index(row, col);
        &mut self.g[index]
    }
}

// Iterator that visits one row or column either forwards or backwards
enum GridAxisIterator<T1, T2, T3, T4> {
    RowLeftToRight(T1),
    RowRightToLeft(T2),
    ColTopToBottom(T3),
    ColBottomToTop(T4),
}

impl<T1, T2, T3, T4> Iterator for GridAxisIterator<T1, T2, T3, T4>
where
    T1: Iterator,
    T2: Iterator<Item = T1::Item>,
    T3: Iterator<Item = T1::Item>,
    T4: Iterator<Item = T1::Item>,
{
    type Item = <T1 as Iterator>::Item;

    fn next(&mut self) -> Option<Self::Item> {
        match self {
            GridAxisIterator::RowLeftToRight(i) => i.next(),
            GridAxisIterator::RowRightToLeft(i) => i.next(),
            GridAxisIterator::ColTopToBottom(i) => i.next(),
            GridAxisIterator::ColBottomToTop(i) => i.next(),
        }
    }
}

// Iterator that visits all rows or columns either forwards or backwards.
// Each item is a row or column iterator.
enum GridIterator<T1, T2, T3, T4> {
    RowsLeftToRight(T1),
    RowsRightToLeft(T2),
    ColsTopToBottom(T3),
    ColsBottomToTop(T4),
}

impl<T1, T2, T3, T4> Iterator for GridIterator<T1, T2, T3, T4>
where
    T1: Iterator,
    T2: Iterator<Item = T1::Item>,
    T3: Iterator<Item = T1::Item>,
    T4: Iterator<Item = T1::Item>,
{
    type Item = <T1 as Iterator>::Item;

    fn next(&mut self) -> Option<Self::Item> {
        match self {
            GridIterator::RowsLeftToRight(i) => i.next(),
            GridIterator::RowsRightToLeft(i) => i.next(),
            GridIterator::ColsTopToBottom(i) => i.next(),
            GridIterator::ColsBottomToTop(i) => i.next(),
        }
    }
}

struct Forest {
    heightmap: Grid,
    visibility_grids: Vec<Grid>,
}

impl Forest {
    fn new(heightmap: Grid) -> Self {
        let rows = heightmap.rows;
        let cols = heightmap.cols;
        Forest {
            heightmap,
            visibility_grids: vec![Grid::new(rows, cols); 4],
        }
    }

    fn compute_visibility_grid_from_each_direction(&mut self) {
        for direction in [
            GridIterDirection::RowsLeftToRight,
            GridIterDirection::RowsRightToLeft,
            GridIterDirection::ColsTopToBottom,
            GridIterDirection::ColsBottomToTop,
        ] {
            let grid_iter = self.heightmap.index_iter_static_dispatch(&direction);
            let direction_index = direction.side_index();
            grid_iter.for_each(|axis_iter| {
                axis_iter.fold(VISIBLE, |mut max_hight, grid_index| {
                    self.visibility_grids[direction_index][grid_index] = max_hight;
                    max_hight = std::cmp::max(max_hight, self.heightmap[grid_index]);
                    max_hight
                });
            });
        }
    }

    fn is_tree_visible(&self, index: Idx) -> bool {
        (0..self.visibility_grids.len())
            .into_iter()
            .map(|side| self.heightmap[index] > self.visibility_grids[side][index])
            .any(|visible| visible)
    }

    fn count_visible_trees(&self) -> usize {
        self.heightmap
            .index_iter_static_dispatch(&GridIterDirection::RowsLeftToRight)
            .flat_map(|axis_iter| axis_iter.map(|index| self.is_tree_visible(index)))
            .filter(|is_visible| *is_visible)
            .count()
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
        self.heightmap.index_iter_static_dispatch(&GridIterDirection::RowsLeftToRight)
        .flat_map(|axis_iter| axis_iter.map(|index| self.get_tree_scenic_score(index.0, index.1)))
        .max().expect("At least one tree should have the highest scenic score")
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
    forest.compute_visibility_grid_from_each_direction();
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
