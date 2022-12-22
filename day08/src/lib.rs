use std::{
    ops::{Index, IndexMut},
    path::Path,
};
use tailsome::IntoResult;

type Height = i8;
const TREE_VISIBLE: Height = -1;

#[derive(Clone, Copy, Debug, derive_more::Display, PartialEq, Eq, Hash, derive_more::From)]
#[display(fmt = "({}, {})", row, col)]
struct GridPos {
    row: usize,
    col: usize,
}

#[derive(Clone, Copy, Debug, derive_more::Display, PartialEq, Eq, Hash, derive_more::From)]
#[display(fmt = "({}, {})", row_delta, col_delta)]
struct GridPosDelta {
    row_delta: isize,
    col_delta: isize,
}

type GridBounds = std::ops::Range<usize>;

#[derive(Clone, Copy, enum_iterator::Sequence, num_enum::IntoPrimitive)]
#[repr(usize)]
enum GridIterDirection {
    Right,
    Left,
    Down,
    Up,
}

impl GridIterDirection {
    fn delta(&self) -> GridPosDelta {
        match self {
            GridIterDirection::Right => (0, 1),
            GridIterDirection::Left => (0, -1),
            GridIterDirection::Down => (1, 0),
            GridIterDirection::Up => (-1, 0),
        }
        .into()
    }
}

#[allow(unused)]
type BoxedAxisIter<'a> = Box<dyn Iterator<Item = &'a Height> + 'a>;
#[allow(unused)]
type BoxedGridIter<'a> = Box<dyn Iterator<Item = BoxedAxisIter<'a>> + 'a>;
type BoxedAxisPosIter<'a> = Box<dyn Iterator<Item = GridPos> + 'a>;
type BoxedGridPosIter<'a> = Box<dyn Iterator<Item = BoxedAxisPosIter<'a>> + 'a>;
type HeightMap = Grid<Height>;

#[derive(Debug, Clone)]
struct Grid<V> {
    g: Vec<V>,
    rows: usize,
    cols: usize,
}

impl<V> Grid<V>
where
    V: Default + Clone,
{
    fn new(rows: usize, cols: usize) -> Self {
        Grid {
            g: vec![V::default(); rows * cols],
            rows,
            cols,
        }
    }
}

impl<V> Grid<V> {
    fn get_element_index(&self, pos: GridPos) -> usize {
        pos.row * self.cols + pos.col
    }

    // Return an pos range (bounds) along an axis, that includes the elements
    // starting from (but excluding) origin into the given direction,
    // until the end of the grid.
    fn bounds_in_direction_of(
        &self,
        direction: &GridIterDirection,
        origin: &GridPos,
    ) -> GridBounds {
        let delta = direction.delta();
        match direction {
            GridIterDirection::Right => {
                origin.col.wrapping_add(delta.col_delta as usize)..self.cols
            }
            GridIterDirection::Left => 0..origin.col,
            GridIterDirection::Down => origin.row.wrapping_add(delta.row_delta as usize)..self.rows,
            GridIterDirection::Up => 0..origin.row,
        }
    }

    // Row major
    fn row_pos_iter_with_bounds(
        &self,
        row: usize,
        bounds: GridBounds,
    ) -> impl DoubleEndedIterator<Item = GridPos> {
        bounds.map(move |col| (row, col).into())
    }

    fn row_pos_iter_rev_with_bounds(
        &self,
        row: usize,
        bounds: GridBounds,
    ) -> impl DoubleEndedIterator<Item = GridPos> {
        self.row_pos_iter_with_bounds(row, bounds).rev()
    }

    fn row_pos_iter(&self, row: usize) -> impl DoubleEndedIterator<Item = GridPos> {
        self.row_pos_iter_with_bounds(row, 0..self.cols)
    }

    fn row_pos_iter_rev(&self, row: usize) -> impl DoubleEndedIterator<Item = GridPos> {
        self.row_pos_iter_rev_with_bounds(row, 0..self.cols)
    }

    // Column major
    fn col_pos_iter_with_bounds(
        &self,
        col: usize,
        bounds: GridBounds,
    ) -> impl DoubleEndedIterator<Item = GridPos> {
        bounds.map(move |row| (row, col).into())
    }

    fn col_pos_iter_rev_with_bounds(
        &self,
        col: usize,
        bounds: GridBounds,
    ) -> impl DoubleEndedIterator<Item = GridPos> {
        self.col_pos_iter_with_bounds(col, bounds).rev()
    }

    fn col_pos_iter(&self, col: usize) -> impl DoubleEndedIterator<Item = GridPos> {
        self.col_pos_iter_with_bounds(col, 0..self.rows)
    }

    fn col_pos_iter_rev(&self, col: usize) -> impl DoubleEndedIterator<Item = GridPos> {
        self.col_pos_iter_rev_with_bounds(col, 0..self.cols)
    }

    #[allow(unused)]
    fn pos_iter_dynamic_dispatch(&self, dir: &GridIterDirection) -> BoxedGridPosIter {
        match dir {
            GridIterDirection::Right => Box::new(
                (0..self.rows).map(|row| Box::new(self.row_pos_iter(row)) as BoxedAxisPosIter),
            ) as BoxedGridPosIter,
            GridIterDirection::Left => Box::new(
                (0..self.rows).map(|row| Box::new(self.row_pos_iter_rev(row)) as BoxedAxisPosIter),
            ) as BoxedGridPosIter,
            GridIterDirection::Down => Box::new(
                (0..self.cols).map(|col| Box::new(self.col_pos_iter(col)) as BoxedAxisPosIter),
            ) as BoxedGridPosIter,
            GridIterDirection::Up => Box::new(
                (0..self.cols).map(|col| Box::new(self.col_pos_iter_rev(col)) as BoxedAxisPosIter),
            ) as BoxedGridPosIter,
        }
    }

    fn pos_iter_along_axis(
        &self,
        axis_index: usize,
        dir: &GridIterDirection,
    ) -> EnumIterator<
        impl Iterator<Item = GridPos>,
        impl Iterator<Item = GridPos>,
        impl Iterator<Item = GridPos>,
        impl Iterator<Item = GridPos>,
    > {
        match dir {
            GridIterDirection::Right => EnumIterator::V1(self.row_pos_iter(axis_index)),
            GridIterDirection::Left => EnumIterator::V2(self.row_pos_iter_rev(axis_index)),
            GridIterDirection::Down => EnumIterator::V3(self.col_pos_iter(axis_index)),
            GridIterDirection::Up => EnumIterator::V4(self.col_pos_iter_rev(axis_index)),
        }
    }

    fn pos_iter_along_axis_with_bounds(
        &self,
        origin: GridPos,
        bounds: GridBounds,
        dir: &GridIterDirection,
    ) -> EnumIterator<
        impl Iterator<Item = GridPos>,
        impl Iterator<Item = GridPos>,
        impl Iterator<Item = GridPos>,
        impl Iterator<Item = GridPos>,
    > {
        match dir {
            GridIterDirection::Right => {
                EnumIterator::V1(self.row_pos_iter_with_bounds(origin.row, bounds))
            }
            GridIterDirection::Left => {
                EnumIterator::V2(self.row_pos_iter_rev_with_bounds(origin.row, bounds))
            }
            GridIterDirection::Down => {
                EnumIterator::V3(self.col_pos_iter_with_bounds(origin.col, bounds))
            }
            GridIterDirection::Up => {
                EnumIterator::V4(self.col_pos_iter_rev_with_bounds(origin.col, bounds))
            }
        }
    }

    fn grid_pos_iter(
        &self,
        dir: &GridIterDirection,
    ) -> impl Iterator<Item = impl Iterator<Item = GridPos>> + '_ {
        let dir = *dir;
        match dir {
            GridIterDirection::Right => {
                EnumIterator::V1((0..self.rows).map(move |row| self.pos_iter_along_axis(row, &dir)))
            }
            GridIterDirection::Left => {
                EnumIterator::V2((0..self.rows).map(move |row| self.pos_iter_along_axis(row, &dir)))
            }
            GridIterDirection::Down => {
                EnumIterator::V3((0..self.cols).map(move |col| self.pos_iter_along_axis(col, &dir)))
            }
            GridIterDirection::Up => {
                EnumIterator::V4((0..self.cols).map(move |col| self.pos_iter_along_axis(col, &dir)))
            }
        }
    }
}

impl<V> std::fmt::Display for Grid<V>
where
    V: std::fmt::Display,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        (0..self.rows).try_for_each(|row| {
            (0..self.cols).try_for_each(|col| write!(f, "{}", self[(row, col).into()]))?;
            writeln!(f)
        })
    }
}

impl<V> Index<GridPos> for Grid<V> {
    type Output = V;

    fn index(&self, pos: GridPos) -> &Self::Output {
        let index = self.get_element_index(pos);
        &self.g[index]
    }
}

impl<V> IndexMut<GridPos> for Grid<V> {
    fn index_mut(&mut self, pos: GridPos) -> &mut Self::Output {
        let index = self.get_element_index(pos);
        &mut self.g[index]
    }
}

// Wrapper enum for iterator static dispatch.
// Could use auto_enum derive instead.
enum EnumIterator<T1, T2, T3, T4> {
    V1(T1),
    V2(T2),
    V3(T3),
    V4(T4),
}

impl<T1, T2, T3, T4> Iterator for EnumIterator<T1, T2, T3, T4>
where
    T1: Iterator,
    T2: Iterator<Item = T1::Item>,
    T3: Iterator<Item = T1::Item>,
    T4: Iterator<Item = T1::Item>,
{
    type Item = <T1 as Iterator>::Item;

    fn next(&mut self) -> Option<Self::Item> {
        match self {
            EnumIterator::V1(i) => i.next(),
            EnumIterator::V2(i) => i.next(),
            EnumIterator::V3(i) => i.next(),
            EnumIterator::V4(i) => i.next(),
        }
    }
}

struct Forest {
    heightmap: HeightMap,
    visibility_grids: Vec<HeightMap>,
}

impl Forest {
    fn new(heightmap: HeightMap) -> Self {
        let rows = heightmap.rows;
        let cols = heightmap.cols;
        Forest {
            heightmap,
            visibility_grids: vec![Grid::new(rows, cols); 4],
        }
    }

    fn compute_visibility_grid_from_each_direction(&mut self) {
        enum_iterator::all::<GridIterDirection>().for_each(|direction| {
            let grid_iter = self.heightmap.grid_pos_iter(&direction);
            let direction_index: usize = direction.into();
            grid_iter.for_each(|axis_iter| {
                axis_iter.fold(TREE_VISIBLE, |mut max_height, pos| {
                    self.visibility_grids[direction_index][pos] = max_height;
                    max_height = std::cmp::max(max_height, self.heightmap[pos]);
                    max_height
                });
            });
        })
    }

    fn is_tree_visible(&self, pos: GridPos) -> bool {
        (0..self.visibility_grids.len())
            .into_iter()
            .map(|direction_index| {
                self.heightmap[pos] > self.visibility_grids[direction_index][pos]
            })
            .any(|visible| visible)
    }

    fn count_visible_trees(&self) -> usize {
        self.heightmap
            .grid_pos_iter(&GridIterDirection::Right)
            .flat_map(|axis_iter| axis_iter.map(|pos| self.is_tree_visible(pos)))
            .filter(|is_visible| *is_visible)
            .count()
    }

    fn get_tree_scenic_score(&self, origin: GridPos) -> usize {
        let tree_height = self.heightmap[origin];

        enum_iterator::all::<GridIterDirection>()
            .map(|direction| {
                let bounds = self.heightmap.bounds_in_direction_of(&direction, &origin);

                // Flag to mark that we need to include the element that is bigger or equal,
                // once we find one. No fancy itertools iterator exists as of commit time.
                let mut took_gte_height = false;
                self.heightmap
                    .pos_iter_along_axis_with_bounds(origin, bounds, &direction)
                    .take_while(move |pos| {
                        if took_gte_height {
                            return false;
                        }
                        if self.heightmap[*pos] >= tree_height {
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
            .grid_pos_iter(&GridIterDirection::Right)
            .flat_map(|axis_iter| axis_iter.map(|pos| self.get_tree_scenic_score(pos)))
            .max()
            .expect("At least one tree should have the highest scenic score")
    }
}

fn parse_grid(s: &str) -> Result<HeightMap, Error> {
    let row_count = s.split('\n').count();

    s.split('\n')
        .enumerate()
        .flat_map(|(row, l)| {
            l.chars().enumerate().map(move |(col, c)| {
                c.to_digit(10)
                    .ok_or(Error::InvalidHeight(c))
                    .map(|height| (row, col, height as Height))
            })
        })
        .try_fold(Grid::new(row_count, row_count), |mut grid, t| {
            let (row, col, h) = t?;
            grid[(row, col).into()] = h;
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
