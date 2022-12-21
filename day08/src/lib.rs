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
    LeftToRight,
    RightToLeft,
    TopToBottom,
    BottomToTop,
}

impl GridIterDirection {
    fn side_index(&self) -> usize {
        match self {
            GridIterDirection::LeftToRight => 0,
            GridIterDirection::RightToLeft => 1,
            GridIterDirection::TopToBottom => 2,
            GridIterDirection::BottomToTop => 3,
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

    // Row major
    fn row_index_iter_with_bounds(
        &self,
        row: usize,
        start_col: usize,
        end_col: usize,
    ) -> impl DoubleEndedIterator<Item = Idx> {
        (start_col..end_col).map(move |col| (row, col))
    }

    fn row_index_iter_rev_with_bounds(
        &self,
        row: usize,
        start_col: usize,
        end_col: usize,
    ) -> impl DoubleEndedIterator<Item = Idx> {
        self.row_index_iter_with_bounds(row, start_col, end_col)
            .rev()
    }

    fn row_index_iter(&self, row: usize) -> impl DoubleEndedIterator<Item = Idx> {
        self.row_index_iter_with_bounds(row, 0, self.cols)
    }

    fn row_index_iter_rev(&self, row: usize) -> impl DoubleEndedIterator<Item = Idx> {
        self.row_index_iter_rev_with_bounds(row, 0, self.cols)
    }

    // Column major
    fn col_index_iter_with_bounds(
        &self,
        col: usize,
        start_row: usize,
        end_row: usize,
    ) -> impl DoubleEndedIterator<Item = Idx> {
        (start_row..end_row).map(move |row| (row, col))
    }

    fn col_index_iter_rev_with_bounds(
        &self,
        col: usize,
        start_row: usize,
        end_row: usize,
    ) -> impl DoubleEndedIterator<Item = Idx> {
        self.col_index_iter_with_bounds(col, start_row, end_row)
            .rev()
    }

    fn col_index_iter(&self, col: usize) -> impl DoubleEndedIterator<Item = Idx> {
        self.col_index_iter_with_bounds(col, 0, self.rows)
    }

    fn col_index_iter_rev(&self, col: usize) -> impl DoubleEndedIterator<Item = Idx> {
        self.col_index_iter_rev_with_bounds(col, 0, self.cols)
    }

    #[allow(unused)]
    fn index_iter_dynamic_dispatch(&self, dir: &GridIterDirection) -> BoxedGridIndexIter {
        match dir {
            GridIterDirection::LeftToRight => Box::new(
                (0..self.rows).map(|row| Box::new(self.row_index_iter(row)) as BoxedAxisIndexIter),
            ) as BoxedGridIndexIter,
            GridIterDirection::RightToLeft => Box::new(
                (0..self.rows)
                    .map(|row| Box::new(self.row_index_iter_rev(row)) as BoxedAxisIndexIter),
            ) as BoxedGridIndexIter,
            GridIterDirection::TopToBottom => Box::new(
                (0..self.cols).map(|col| Box::new(self.col_index_iter(col)) as BoxedAxisIndexIter),
            ) as BoxedGridIndexIter,
            GridIterDirection::BottomToTop => Box::new(
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
            GridIterDirection::LeftToRight => {
                GridAxisIterator::RowLeftToRight(self.row_index_iter(axis_index))
            }
            GridIterDirection::RightToLeft => {
                GridAxisIterator::RowRightToLeft(self.row_index_iter_rev(axis_index))
            }
            GridIterDirection::TopToBottom => {
                GridAxisIterator::ColTopToBottom(self.col_index_iter(axis_index))
            }
            GridIterDirection::BottomToTop => {
                GridAxisIterator::ColBottomToTop(self.col_index_iter_rev(axis_index))
            }
        }
    }

    fn axis_index_iter_with_bounds(
        &self,
        axis_index: usize,
        start_bound: usize,
        end_bound: usize,
        dir: &GridIterDirection,
    ) -> GridAxisIterator<
        impl Iterator<Item = Idx>,
        impl Iterator<Item = Idx>,
        impl Iterator<Item = Idx>,
        impl Iterator<Item = Idx>,
    > {
        match dir {
            GridIterDirection::LeftToRight => GridAxisIterator::RowLeftToRight(
                self.row_index_iter_with_bounds(axis_index, start_bound, end_bound),
            ),
            GridIterDirection::RightToLeft => GridAxisIterator::RowRightToLeft(
                self.row_index_iter_rev_with_bounds(axis_index, start_bound, end_bound),
            ),
            GridIterDirection::TopToBottom => GridAxisIterator::ColTopToBottom(
                self.col_index_iter_with_bounds(axis_index, start_bound, end_bound),
            ),
            GridIterDirection::BottomToTop => GridAxisIterator::ColBottomToTop(
                self.col_index_iter_rev_with_bounds(axis_index, start_bound, end_bound),
            ),
        }
    }

    fn index_iter_static_dispatch(
        &self,
        dir: &GridIterDirection,
    ) -> impl Iterator<Item = impl Iterator<Item = Idx>> + '_ {
        let dir = *dir;
        match dir {
            GridIterDirection::LeftToRight => GridIterator::RowsLeftToRight(
                (0..self.rows).map(move |row| self.axis_index_iter_static(row, &dir)),
            ),
            GridIterDirection::RightToLeft => GridIterator::RowsRightToLeft(
                (0..self.rows).map(move |row| self.axis_index_iter_static(row, &dir)),
            ),
            GridIterDirection::TopToBottom => GridIterator::ColsTopToBottom(
                (0..self.cols).map(move |col| self.axis_index_iter_static(col, &dir)),
            ),
            GridIterDirection::BottomToTop => GridIterator::ColsBottomToTop(
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
        [
            GridIterDirection::LeftToRight,
            GridIterDirection::RightToLeft,
            GridIterDirection::TopToBottom,
            GridIterDirection::BottomToTop,
        ]
        .into_iter()
        .for_each(|direction| {
            let grid_iter = self.heightmap.index_iter_static_dispatch(&direction);
            let direction_index = direction.side_index();
            grid_iter.for_each(|axis_iter| {
                axis_iter.fold(VISIBLE, |mut max_hight, grid_index| {
                    self.visibility_grids[direction_index][grid_index] = max_hight;
                    max_hight = std::cmp::max(max_hight, self.heightmap[grid_index]);
                    max_hight
                });
            });
        })
    }

    fn is_tree_visible(&self, index: Idx) -> bool {
        (0..self.visibility_grids.len())
            .into_iter()
            .map(|side| self.heightmap[index] > self.visibility_grids[side][index])
            .any(|visible| visible)
    }

    fn count_visible_trees(&self) -> usize {
        self.heightmap
            .index_iter_static_dispatch(&GridIterDirection::LeftToRight)
            .flat_map(|axis_iter| axis_iter.map(|index| self.is_tree_visible(index)))
            .filter(|is_visible| *is_visible)
            .count()
    }

    fn get_tree_scenic_score(&self, index: Idx) -> usize {
        let row = index.0;
        let col = index.1;
        let tree_height = self.heightmap[(row, col)];

        let directions = [
            (
                GridIterDirection::LeftToRight,
                row,
                col + 1,
                self.heightmap.cols,
            ),
            (GridIterDirection::RightToLeft, row, 0, col),
            (
                GridIterDirection::TopToBottom,
                col,
                row + 1,
                self.heightmap.rows,
            ),
            (GridIterDirection::BottomToTop, col, 0, row),
        ];

        directions
            .into_iter()
            .map(|(direction, axis_index, start_bound, end_bound)| {
                // Flag to check that we need to include the element that is bigger or equal.
                // once we find one. No fancy itertools iterator exists as of commit time.
                let mut took_gte_height = false;
                self.heightmap
                    .axis_index_iter_with_bounds(axis_index, start_bound, end_bound, &direction)
                    .take_while(move |index| {
                        if took_gte_height {
                            return false;
                        }
                        if self.heightmap[*index] >= tree_height {
                            took_gte_height = true;
                        }
                        true
                    })
                    .count()
            })
            .product()
    }

    fn find_highest_scenic_score(&self) -> usize {
        self.heightmap
            .index_iter_static_dispatch(&GridIterDirection::LeftToRight)
            .flat_map(|axis_iter| axis_iter.map(|index| self.get_tree_scenic_score(index)))
            .max()
            .expect("At least one tree should have the highest scenic score")
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
