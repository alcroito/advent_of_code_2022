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

struct IterWrapper<T>(T);

impl<T> Iterator for IterWrapper<T>
where
    T: Iterator,
{
    type Item = <T as ::core::iter::Iterator>::Item;

    fn next(&mut self) -> Option<Self::Item> {
        self.0.next()
    }
}

impl<T> DoubleEndedIterator for IterWrapper<T>
where
    T: DoubleEndedIterator,
{
    fn next_back(&mut self) -> Option<Self::Item> {
        self.0.next_back()
    }
}

#[allow(unused)]
type GridRowIterType<'a> = std::slice::Iter<'a, Value>;
#[allow(unused)]
type GridRowIterRevType<'a> = std::iter::Rev<std::slice::Iter<'a, Value>>;
#[allow(unused)]
type GridRowIterDirectedType<'a> = GridAxisIter<GridRowIterType<'a>, GridRowIterRevType<'a>>;
type GridColIterType<'a, T> = IterWrapper<T>;
type GridColIterRevType<'a, T> = std::iter::Rev<IterWrapper<T>>;
type GridColIterDirectedType<'a, T1, T2> =
    GridAxisIter<GridColIterType<'a, T1>, GridColIterRevType<'a, T2>>;

#[allow(unused)]
type BoxedAxisIter<'a> = Box<dyn Iterator<Item = &'a Value> + 'a>;
#[allow(unused)]
type BoxedGridIter<'a> = Box<dyn Iterator<Item = BoxedAxisIter<'a>> + 'a>;

type BoxedAxisIndexIter<'a> = Box<dyn Iterator<Item = Idx> + 'a>;
type BoxedGridIndexIter<'a> = Box<dyn Iterator<Item = BoxedAxisIndexIter<'a>> + 'a>;

impl Grid {
    fn new(rows: usize, cols: usize) -> Self {
        Grid {
            g: vec![VISIBLE; rows * cols],
            rows,
            cols,
        }
    }

    fn get_index(&self, row: usize, col: usize) -> usize {
        row * self.cols + col
    }

    #[allow(unused)]
    fn row_iter(&self, row: usize) -> GridRowIterType<'_> {
        let start = self.get_index(row, 0);
        let end = start + self.cols;
        self.g[start..end].iter()
    }

    fn row_index_iter(&self, row: usize) -> impl DoubleEndedIterator<Item = Idx> {
        let start = 0;
        let end = self.cols;
        (start..end).map(move |col| (row, col))
    }

    #[allow(unused)]
    fn row_iter_rev(&self, row: usize) -> GridRowIterRevType<'_> {
        self.row_iter(row).rev()
    }

    fn row_index_iter_rev(&self, row: usize) -> impl DoubleEndedIterator<Item = Idx> {
        self.row_index_iter(row).rev()
    }

    #[allow(unused)]
    fn row_iter_directed(
        &self,
        row: usize,
        direction: &GridAxisDirection,
    ) -> GridRowIterDirectedType<'_> {
        match direction {
            GridAxisDirection::Forward => GridAxisIter::Forward(self.row_iter(row)),
            GridAxisDirection::Reverse => GridAxisIter::Reverse(self.row_iter_rev(row)),
        }
    }

    // fn rows_iter(&self) -> impl DoubleEndedIterator<Item = GridRowIterDirectedType<'_>> {
    //     (0..self.rows).map(|row| self.row_iter_directed(row, &GridAxisDirection::Forward))
    // }

    // fn rows_iter_rev(&self) -> impl DoubleEndedIterator<Item = GridRowIterDirectedType<'_>> {
    //     (0..self.rows).map(|row| self.row_iter_directed(row, &GridAxisDirection::Reverse))
    // }

    #[allow(unused)]
    fn col_iter(
        &self,
        col: usize,
    ) -> GridColIterType<'_, impl DoubleEndedIterator<Item = &Value> + '_> {
        let start = 0;
        let end = self.rows;
        IterWrapper((start..end).map(move |row| &self[(row, col)]))
    }

    fn col_index_iter(&self, col: usize) -> impl DoubleEndedIterator<Item = Idx> {
        let start = 0;
        let end = self.rows;
        (start..end).map(move |row| (row, col))
    }

    #[allow(unused)]
    fn col_iter_rev(
        &self,
        col: usize,
    ) -> GridColIterRevType<'_, impl DoubleEndedIterator<Item = &Value> + '_> {
        self.col_iter(col).rev()
    }

    fn col_index_iter_rev(&self, col: usize) -> impl DoubleEndedIterator<Item = Idx> {
        self.col_index_iter(col).rev()
    }

    #[allow(unused)]
    fn col_iter_directed(
        &self,
        col: usize,
        direction: &GridAxisDirection,
    ) -> GridColIterDirectedType<
        '_,
        impl DoubleEndedIterator<Item = &Value> + '_,
        impl DoubleEndedIterator<Item = &Value> + '_,
    > {
        match direction {
            GridAxisDirection::Forward => GridAxisIter::Forward(self.col_iter(col)),
            GridAxisDirection::Reverse => GridAxisIter::Reverse(self.col_iter_rev(col)),
        }
    }

    // fn cols_iter(&self) -> impl DoubleEndedIterator<Item = GridColIterDirectedType<'_, impl DoubleEndedIterator + '_, impl DoubleEndedIterator + '_>> {
    //     (0..self.cols).map(|col| self.col_iter_directed(col, &GridAxisDirection::Forward))
    // }

    // fn cols_iter_rev(&self) -> impl DoubleEndedIterator<Item = GridColIterDirectedType<'_, impl DoubleEndedIterator + '_, impl DoubleEndedIterator + '_>> {
    //     (0..self.cols).map(|col| self.col_iter_directed(col, &GridAxisDirection::Reverse))
    // }

    // fn generic_iter(&self, dir: &RayDirection) ->
    //     GridIterator<
    //     impl Iterator + '_,
    //     impl Iterator + '_,
    //     impl Iterator + '_,
    //     impl Iterator + '_,
    //     > {
    //     match dir {
    //         RayDirection::LeftToRight => GridIterator::LeftToRight({
    //             (0..self.rows).map(|row| self.row_iter_directed(row, &GridAxisDirection::Forward))
    //         }),
    //         RayDirection::RightToLeft => GridIterator::RightToLeft({
    //             (0..self.rows).map(|row| self.row_iter_directed(row, &GridAxisDirection::Reverse))
    //         }),
    //         RayDirection::TopToBottom => GridIterator::TopToBottom({
    //             (0..self.cols).map(|col| self.col_iter_directed(col, &GridAxisDirection::Forward))
    //         }),
    //         RayDirection::BottomToTop => GridIterator::BottomToTop({
    //             (0..self.cols).map(|col| self.col_iter_directed(col, &GridAxisDirection::Reverse))
    //         }),
    //     }
    // }

    // fn cast(&self) {
    //     let a = self.row_iter_directed(5, &GridAxisDirection::Forward);
    //     let b = Box::new(a) as BoxedAxisIter;
    //     let c = (0..self.rows).map(|row| Box::new(self.row_iter_directed(row, &GridAxisDirection::Forward)) as BoxedAxisIter);
    //     let d = Box::new(c) as BoxedGridIter;
    // }

    // fn generic_iter(&self, dir: &RayDirection) -> BoxedGridIter {
    //     match dir {
    //         RayDirection::LeftToRight => Box::new({
    //             (0..self.rows).map(|row| {
    //                 Box::new(self.row_iter_directed(row, &GridAxisDirection::Forward))
    //                     as BoxedAxisIter
    //             })
    //         }) as BoxedGridIter,
    //         RayDirection::RightToLeft => Box::new({
    //             (0..self.rows).map(|row| {
    //                 Box::new(self.row_iter_directed(row, &GridAxisDirection::Reverse))
    //                     as BoxedAxisIter
    //             })
    //         }) as BoxedGridIter,
    //         RayDirection::TopToBottom => Box::new({
    //             (0..self.cols).map(|col| {
    //                 Box::new(self.col_iter_directed(col, &GridAxisDirection::Forward))
    //                     as BoxedAxisIter
    //             })
    //         }) as BoxedGridIter,
    //         RayDirection::BottomToTop => Box::new({
    //             (0..self.cols).map(|col| {
    //                 Box::new(self.col_iter_directed(col, &GridAxisDirection::Reverse))
    //                     as BoxedAxisIter
    //             })
    //         }) as BoxedGridIter,
    //     }
    // }

    fn index_iter(&self, dir: &GridIterDirection) -> BoxedGridIndexIter {
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

    #[allow(unused)]
    fn axis_index_iter(
        &self,
        axis: usize,
        dir: &GridIterDirection,
    ) -> GridIteratorAxis<
        impl Iterator<Item = Idx>,
        impl Iterator<Item = Idx>,
        impl Iterator<Item = Idx>,
        impl Iterator<Item = Idx>,
    > {
        match dir {
            GridIterDirection::LeftToRight => {
                GridIteratorAxis::LeftToRight(self.row_index_iter(axis))
            }
            GridIterDirection::RightToLeft => {
                GridIteratorAxis::RightToLeft(self.row_index_iter_rev(axis))
            }
            GridIterDirection::TopToBottom => {
                GridIteratorAxis::TopToBottom(self.col_index_iter(axis))
            }
            GridIterDirection::BottomToTop => {
                GridIteratorAxis::BottomToTop(self.col_index_iter_rev(axis))
            }
        }
    }

    #[allow(unused)]
    fn index_iter_static_dispatch(&self, dir: &GridIterDirection) -> impl Iterator + '_ {
        let dir = *dir;
        match dir {
            GridIterDirection::LeftToRight => GridIterator::LeftToRight(
                (0..self.rows).map(move |row| self.axis_index_iter(row, &dir)),
            ),
            GridIterDirection::RightToLeft => GridIterator::RightToLeft(
                (0..self.rows).map(move |row| self.axis_index_iter(row, &dir)),
            ),
            GridIterDirection::TopToBottom => GridIterator::TopToBottom(
                (0..self.cols).map(move |col| self.axis_index_iter(col, &dir)),
            ),
            GridIterDirection::BottomToTop => GridIterator::BottomToTop(
                (0..self.cols).map(move |col| self.axis_index_iter(col, &dir)),
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
        let index = self.get_index(row, col);
        &self.g[index]
    }
}

impl IndexMut<Idx> for Grid {
    fn index_mut(&mut self, index: Idx) -> &mut Self::Output {
        let row = index.0;
        let col = index.1;
        let index = self.get_index(row, col);
        &mut self.g[index]
    }
}

#[allow(unused)]
enum GridAxisDirection {
    Forward,
    Reverse,
}

enum GridAxisIter<T1, T2> {
    Forward(T1),
    Reverse(T2),
}

impl<'a, T1, T2> Iterator for GridAxisIter<T1, T2>
where
    T1: DoubleEndedIterator<Item = &'a Value>,
    T2: DoubleEndedIterator<Item = &'a Value>,
{
    type Item = <T1 as ::core::iter::Iterator>::Item;

    fn next(&mut self) -> Option<Self::Item> {
        match self {
            GridAxisIter::Forward(i) => i.next(),
            GridAxisIter::Reverse(i) => i.next(),
        }
    }
}

// enum GridGenericIter<T1, T2> {
//     RowIter(T1),
//     ColIter(T2)
// }

// impl<'a, T1, T2> Iterator for GridGenericIter<T1, T2>
// where
// T1: DoubleEndedIterator<Item = &'a Value>,
// T2: DoubleEndedIterator<Item = &'a Value>
// {
//     type Item = <T1 as ::core::iter::Iterator>::Item;

//     fn next(&mut self) -> Option<Self::Item> {
//         match self {
//             GridGenericIter::RowIter(i) => i.next(),
//             GridGenericIter::ColIter(i) => i.next(),
//         }
//     }
// }

enum GridIteratorAxis<T1, T2, T3, T4> {
    LeftToRight(T1),
    RightToLeft(T2),
    TopToBottom(T3),
    BottomToTop(T4),
}

impl<T1, T2, T3, T4> Iterator for GridIteratorAxis<T1, T2, T3, T4>
where
    T1: Iterator<Item = Idx>,
    T2: Iterator<Item = Idx>,
    T3: Iterator<Item = Idx>,
    T4: Iterator<Item = Idx>,
{
    type Item = <T1 as ::core::iter::Iterator>::Item;

    fn next(&mut self) -> Option<Self::Item> {
        match self {
            GridIteratorAxis::LeftToRight(i) => i.next(),
            GridIteratorAxis::RightToLeft(i) => i.next(),
            GridIteratorAxis::TopToBottom(i) => i.next(),
            GridIteratorAxis::BottomToTop(i) => i.next(),
        }
    }
}

enum GridIterator<T1, T2, T3, T4> {
    LeftToRight(T1),
    RightToLeft(T2),
    TopToBottom(T3),
    BottomToTop(T4),
}

impl<T1, T2, T3, T4, I1, I2, I3, I4> Iterator for GridIterator<T1, T2, T3, T4>
where
    T1: Iterator<Item = GridIteratorAxis<I1, I2, I3, I4>>,
    T2: Iterator<Item = GridIteratorAxis<I1, I2, I3, I4>>,
    T3: Iterator<Item = GridIteratorAxis<I1, I2, I3, I4>>,
    T4: Iterator<Item = GridIteratorAxis<I1, I2, I3, I4>>,
    // I1: Iterator<Item = Idx>,
    // I2: Iterator<Item = Idx>,
    // I3: Iterator<Item = Idx>,
    // I4: Iterator<Item = Idx>
{
    type Item = <T1 as ::core::iter::Iterator>::Item;

    fn next(&mut self) -> Option<Self::Item> {
        match self {
            GridIterator::LeftToRight(i) => i.next(),
            GridIterator::RightToLeft(i) => i.next(),
            GridIterator::TopToBottom(i) => i.next(),
            GridIterator::BottomToTop(i) => i.next(),
        }
    }
}

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

    // fn direction_iter(&self, dir: &RayDirection) -> (std::ops::Range<usize>,
    //     std::iter::Chain<std::ops::Range<usize>, std::iter::Rev<std::ops::Range<usize>>>)  {
    //     match dir {
    //         RayDirection::LeftToRight => (0..self.heightmap.rows, (0..self.heightmap.cols).chain((0..0).rev())),
    //         RayDirection::RightToLeft => (0..self.heightmap.rows, (0..0).chain((0..self.heightmap.cols).rev())),
    //         RayDirection::TopToBottom => (0..self.heightmap.cols, (0..self.heightmap.rows).chain((0..0).rev())),
    //         RayDirection::BottomToTop => (0..self.heightmap.cols, (0..0).chain((0..self.heightmap.rows).rev())),
    //     }
    // }

    // #[auto_enums::auto_enum(Iterator)]
    // fn direction_iter(&self, dir: &RayDirection) -> impl DoubleEndedIterator<Item = impl DoubleEndedIterator<Item = &Value>> {
    //     match dir {
    //         RayDirection::LeftToRight => self.heightmap.rows_iter(),
    //         RayDirection::RightToLeft => self.heightmap.rows_iter_rev(),
    //         RayDirection::TopToBottom => self.heightmap.cols_iter(),
    //         RayDirection::BottomToTop => self.heightmap.cols_iter_rev(),
    //     }
    // }

    fn compute_visibility(&mut self) {
        for side in [
            GridIterDirection::LeftToRight,
            GridIterDirection::RightToLeft,
            GridIterDirection::TopToBottom,
            GridIterDirection::BottomToTop,
        ] {
            let iter = self.heightmap.index_iter(&side);
            let side = side.side_index();
            for main_axis in iter {
                let mut highest = VISIBLE;
                for index in main_axis {
                    self.cover_sides[side][index] = highest;
                    highest = std::cmp::max(highest, self.heightmap[index]);
                }
            }
        }

        // for side in [RayDirection::LeftToRight, RayDirection::RightToLeft, RayDirection::TopToBottom, RayDirection::BottomToTop] {
        //     let (row_range, col_range) = self.direction_iter(&side);
        //     let side = side.side_index();
        //     for r in row_range {
        //         let mut highest = VISIBLE;
        //         let col_range = col_range.clone();
        //         for c in col_range {
        //             self.cover_sides[side][(r, c)] = highest;
        //             highest = std::cmp::max(highest, self.heightmap[(r, c)]);
        //         }
        //     }
        // }

        // let side = RayDirection::LeftToRight.side_index();
        // for r in 0..self.heightmap.rows {
        //     let mut highest = VISIBLE;
        //     for c in 0..self.heightmap.cols {
        //         self.cover_sides[side][(r, c)] = highest;
        //         highest = std::cmp::max(highest, self.heightmap[(r, c)]);
        //     }
        // }

        // let side = RayDirection::RightToLeft.side_index();
        // for r in 0..self.heightmap.rows {
        //     let mut highest = VISIBLE;
        //     for c in (0..self.heightmap.cols).rev() {
        //         self.cover_sides[side][(r, c)] = highest;
        //         highest = std::cmp::max(highest, self.heightmap[(r, c)]);
        //     }
        // }

        // let side = RayDirection::TopToBottom.side_index();
        // for c in 0..self.heightmap.cols {
        //     let mut highest = VISIBLE;
        //     for r in 0..self.heightmap.rows {
        //         self.cover_sides[side][(r, c)] = highest;
        //         highest = std::cmp::max(highest, self.heightmap[(r, c)]);
        //     }
        // }

        // let side = RayDirection::BottomToTop.side_index();
        // for c in 0..self.heightmap.cols {
        //     let mut highest = VISIBLE;
        //     for r in (0..self.heightmap.rows).rev() {
        //         self.cover_sides[side][(r, c)] = highest;
        //         highest = std::cmp::max(highest, self.heightmap[(r, c)]);
        //     }
        // }
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
