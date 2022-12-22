use helpers::grid::{Grid, GridIterDirection, GridPos};
use std::path::Path;
use tailsome::IntoResult;

type Height = i8;
const TREE_VISIBLE: Height = -1;
type HeightMap = Grid<Height>;

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
