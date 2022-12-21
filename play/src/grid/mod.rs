type Value = i8;
type Idx = (usize, usize);

#[derive(Clone, Copy)]
enum GridIterDirection {
    RowsLeftToRight,
    RowsRightToLeft,
    ColsTopToBottom,
    ColsBottomToTop,
}

impl std::fmt::Display for GridIterDirection {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            GridIterDirection::RowsLeftToRight => write!(f, "RowsLeftToRight"),
            GridIterDirection::RowsRightToLeft => write!(f, "RowsRightToLeft"),
            GridIterDirection::ColsTopToBottom => write!(f, "ColsTopToBottom"),
            GridIterDirection::ColsBottomToTop => write!(f, "ColsBottomToTop"),
        }
    }
}

#[derive(Debug, Clone)]
struct Grid {
    #[allow(unused)]
    g: Vec<Value>,
    rows: usize,
    cols: usize,
}

impl Grid {
    fn new(rows: usize, cols: usize) -> Self {
        Grid {
            g: vec![0; rows * cols],
            rows,
            cols,
        }
    }

    #[allow(unused)]
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

    #[allow(unused)]
    fn axis_index_iter(
        &self,
        axis: usize,
        dir: &GridIterDirection,
    ) -> GridAxisIterator<
        impl Iterator<Item = Idx>,
        impl Iterator<Item = Idx>,
        impl Iterator<Item = Idx>,
        impl Iterator<Item = Idx>,
    > {
        match dir {
            GridIterDirection::RowsLeftToRight => {
                GridAxisIterator::RowLeftToRight(self.row_index_iter(axis))
            }
            GridIterDirection::RowsRightToLeft => {
                GridAxisIterator::RowRightToLeft(self.row_index_iter_rev(axis))
            }
            GridIterDirection::ColsTopToBottom => {
                GridAxisIterator::ColTopToBottom(self.col_index_iter(axis))
            }
            GridIterDirection::ColsBottomToTop => {
                GridAxisIterator::ColBottomToTop(self.col_index_iter_rev(axis))
            }
        }
    }

    #[allow(unused)]
    fn index_iter_static_dispatch(
        &self,
        dir: &GridIterDirection,
    ) -> impl Iterator<Item = impl Iterator<Item = Idx>> + '_ {
        let dir = *dir;
        match dir {
            GridIterDirection::RowsLeftToRight => GridIterator::RowsLeftToRight(
                (0..self.rows).map(move |row| self.axis_index_iter(row, &dir)),
            ),
            GridIterDirection::RowsRightToLeft => GridIterator::RowsRightToLeft(
                (0..self.rows).map(move |row| self.axis_index_iter(row, &dir)),
            ),
            GridIterDirection::ColsTopToBottom => GridIterator::ColsTopToBottom(
                (0..self.cols).map(move |col| self.axis_index_iter(col, &dir)),
            ),
            GridIterDirection::ColsBottomToTop => GridIterator::ColsBottomToTop(
                (0..self.cols).map(move |col| self.axis_index_iter(col, &dir)),
            ),
        }
    }
}

// Boxed type aliases for dynamic dispatch
type BoxedAxisIndexIter<'a> = Box<dyn Iterator<Item = Idx> + 'a>;
type BoxedGridIndexIter<'a> = Box<dyn Iterator<Item = BoxedAxisIndexIter<'a>> + 'a>;

// Custom enums for static dispatch
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

pub fn iterate_grid() {
    let grid = Grid::new(3, 3);

    for dir in [
        GridIterDirection::RowsLeftToRight,
        GridIterDirection::RowsRightToLeft,
        GridIterDirection::ColsTopToBottom,
        GridIterDirection::ColsBottomToTop,
    ] {
        // let iter = grid.index_iter_dynamic_dispatch(&dir);
        let iter = grid.index_iter_static_dispatch(&dir);

        println!("{}", dir);
        for main_axis in iter {
            for index in main_axis {
                let value = grid.g[grid.get_element_index(index.0, index.1)];
                println!("{:?}, {}", index, value);
            }
        }
    }
}
