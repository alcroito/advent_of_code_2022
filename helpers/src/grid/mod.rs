use std::ops::{Index, IndexMut};

#[derive(Clone, Copy, Debug, Default, derive_more::Display, PartialEq, Eq, Hash, derive_more::From, derive_more::Into)]
#[display(fmt = "({}, {})", row, col)]
pub struct GridPos {
    pub row: usize,
    pub col: usize,
}

impl std::ops::Add<GridPosDelta> for GridPos {
    type Output = GridPos;

    fn add(self, rhs: GridPosDelta) -> Self::Output {
        let row = (self.row as isize + rhs.row_delta) as usize;
        let col = (self.col as isize + rhs.col_delta) as usize;
        (row, col).into()
    }
}

impl std::ops::Sub<GridPos> for GridPos {
    type Output = GridPosDelta;

    fn sub(self, rhs: GridPos) -> Self::Output {
        let row = self.row as isize - rhs.row as isize;
        let col = self.col as isize - rhs.col as isize;
        (row, col).into()
    }
}

impl std::ops::AddAssign<GridPosDelta> for GridPos {
    fn add_assign(&mut self, rhs: GridPosDelta) {
        *self = *self + rhs;
    }
}

#[derive(Clone, Copy, Debug, Default, derive_more::Display, PartialEq, Eq, Hash, derive_more::From, derive_more::Into)]
#[display(fmt = "({}, {})", row, col)]
pub struct GridPosISize {
    pub row: isize,
    pub col: isize,
}

impl std::ops::Add<GridPosDelta> for GridPosISize {
    type Output = GridPosISize;

    fn add(self, rhs: GridPosDelta) -> Self::Output {
        let row = self.row + rhs.row_delta;
        let col = self.col + rhs.col_delta;
        (row, col).into()
    }
}

impl std::ops::AddAssign<GridPosDelta> for GridPosISize {
    fn add_assign(&mut self, rhs: GridPosDelta) {
        *self = *self + rhs;
    }
}

#[derive(Clone, Copy, Debug, derive_more::Display, PartialEq, Eq, Hash, derive_more::From, derive_more::Into)]
#[display(fmt = "({}, {})", row_delta, col_delta)]
pub struct GridPosDelta {
    pub row_delta: isize,
    pub col_delta: isize,
}

impl std::ops::Mul<isize> for GridPosDelta {
    type Output = GridPosDelta;

    fn mul(self, rhs: isize) -> Self::Output {
        let row = self.row_delta * rhs;
        let col = self.col_delta * rhs;
        (row, col).into()
    }
}


pub type GridBounds = std::ops::Range<usize>;

#[derive(Default, Debug)]
pub struct GridExtents {
    pub row_range: std::ops::Range<isize>,
    pub col_range: std::ops::Range<isize>,
}

impl GridExtents {
    pub fn normalized(&self) -> GridExtents {
        let row_max = self.row_range.start.abs() + self.row_range.end + 1;
        let col_max = self.col_range.start.abs() + self.col_range.end + 1;
        GridExtents {
            row_range: 0..row_max,
            col_range: 0..col_max,
        }
    }

    pub fn normalized_pos(&self, pos: &GridPos) -> GridPos {
        let row_delta = self.row_range.start.unsigned_abs();
        let col_delta = self.col_range.start.unsigned_abs();
        let (row, col) = (*pos).into();
        (row + row_delta, col + col_delta).into()
    }

    pub fn compute_grid_extents<I>(iter: I) -> GridExtents
    where I: Iterator<Item=GridPosISize>
     {
        let mut extents = GridExtents::default();
        iter.for_each(|pos| {
            extents.row_range.start = extents.row_range.start.min(pos.row);
            extents.row_range.end = extents.row_range.end.max(pos.row);
            extents.col_range.start = extents.col_range.start.min(pos.col);
            extents.col_range.end = extents.col_range.end.max(pos.col);
        });
        extents
    }
}

#[derive(Clone, Copy, enum_iterator::Sequence, num_enum::IntoPrimitive)]
#[repr(usize)]
pub enum GridIterDirection {
    Right,
    Left,
    Down,
    Up,
}

impl GridIterDirection {
    pub fn delta(&self) -> GridPosDelta {
        match self {
            GridIterDirection::Right => (0, 1),
            GridIterDirection::Left => (0, -1),
            GridIterDirection::Down => (1, 0),
            GridIterDirection::Up => (-1, 0),
        }
        .into()
    }
}

#[derive(Debug, Clone, Copy, enum_iterator::Sequence)]
pub enum Direction9 {
    UpLeft,
    Up,
    UpRight,
    Right,
    DownRight,
    Down,
    DownLeft,
    Left,
    Center,
}

impl Direction9 {
    pub fn iter() -> enum_iterator::All<Direction9> {
        enum_iterator::all::<Direction9>()
    }

    pub fn next(&self) -> Option<Self> {
        enum_iterator::next(self)
    }

    pub fn delta(&self) -> GridPosDelta {
        match self {
            Self::UpLeft => (-1, -1),
            Self::Up => (-1, 0),
            Self::UpRight => (-1, 1),
            Self::Right => (0, 1),
            Self::DownRight => (1, 1),
            Self::Down => (1, 0),
            Self::DownLeft => (1, -1),
            Self::Left => (0, -1),
            Self::Center => (0, 0),
        }
        .into()
    }
}

type BoxedAxisPosIter<'a> = Box<dyn Iterator<Item = GridPos> + 'a>;
type BoxedGridPosIter<'a> = Box<dyn Iterator<Item = BoxedAxisPosIter<'a>> + 'a>;

#[derive(Debug, Clone)]
pub struct Grid<V> {
    g: Vec<V>,
    pub rows: usize,
    pub cols: usize,
}

impl<V> Grid<V>
where
    V: Default + Clone,
{
    pub fn new(rows: usize, cols: usize) -> Self {
        Grid {
            g: vec![V::default(); rows * cols],
            rows,
            cols,
        }
    }
}

impl<V> Grid<V> {
    pub fn get_element_index(&self, pos: GridPos) -> usize {
        pos.row * self.cols + pos.col
    }
    
    pub fn get_pos_from_linear_index(&self, index: usize) -> GridPos {
        GridPos { row: index / self.cols, col: index % self.cols }
    }

    // Return a pos range (bounds) along an axis, that includes the elements
    // starting from (but excluding) origin into the given direction,
    // until the end of the grid.
    pub fn bounds_in_direction_of(
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
    pub fn row_pos_iter_with_bounds(
        &self,
        row: usize,
        bounds: GridBounds,
    ) -> impl DoubleEndedIterator<Item = GridPos> {
        bounds.map(move |col| (row, col).into())
    }

    pub fn row_pos_iter_rev_with_bounds(
        &self,
        row: usize,
        bounds: GridBounds,
    ) -> impl DoubleEndedIterator<Item = GridPos> {
        self.row_pos_iter_with_bounds(row, bounds).rev()
    }

    pub fn row_pos_iter(&self, row: usize) -> impl DoubleEndedIterator<Item = GridPos> {
        self.row_pos_iter_with_bounds(row, 0..self.cols)
    }

    pub fn row_pos_iter_rev(&self, row: usize) -> impl DoubleEndedIterator<Item = GridPos> {
        self.row_pos_iter_rev_with_bounds(row, 0..self.cols)
    }

    // Column major
    pub fn col_pos_iter_with_bounds(
        &self,
        col: usize,
        bounds: GridBounds,
    ) -> impl DoubleEndedIterator<Item = GridPos> {
        bounds.map(move |row| (row, col).into())
    }

    pub fn col_pos_iter_rev_with_bounds(
        &self,
        col: usize,
        bounds: GridBounds,
    ) -> impl DoubleEndedIterator<Item = GridPos> {
        self.col_pos_iter_with_bounds(col, bounds).rev()
    }

    pub fn col_pos_iter(&self, col: usize) -> impl DoubleEndedIterator<Item = GridPos> {
        self.col_pos_iter_with_bounds(col, 0..self.rows)
    }

    pub fn col_pos_iter_rev(&self, col: usize) -> impl DoubleEndedIterator<Item = GridPos> {
        self.col_pos_iter_rev_with_bounds(col, 0..self.cols)
    }

    pub fn pos_iter_dynamic_dispatch(&self, dir: &GridIterDirection) -> BoxedGridPosIter {
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

    pub fn pos_iter_along_axis(
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

    pub fn pos_iter_along_axis_with_bounds(
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

    pub fn grid_pos_iter(
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
pub enum EnumIterator<T1, T2, T3, T4> {
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
