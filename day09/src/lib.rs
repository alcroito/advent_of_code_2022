use eframe::egui;
use error_stack::{IntoReport, Report, Result as ESResult, ResultExt};
use helpers::grid::{
    Direction9, Grid, GridExtents, GridIterDirection, GridPos, GridPosDelta, GridPosISize,
};
use itertools::Itertools;
use std::{
    path::Path,
    str::FromStr,
    sync::{Arc, Barrier, Mutex},
};
use tailsome::IntoResult;

type StepCount = usize;

#[derive(Clone, Copy, Debug, derive_more::Display, PartialEq, Eq, Hash)]
enum OpKind {
    #[display(fmt = "U")]
    Up,
    #[display(fmt = "R")]
    Right,
    #[display(fmt = "D")]
    Down,
    #[display(fmt = "L")]
    Left,
}

#[derive(Clone, Copy, Debug, derive_more::Display, PartialEq, Eq, Hash)]
#[display(fmt = "{} {}", op_kind, step_count)]
struct Op {
    op_kind: OpKind,
    step_count: StepCount,
}

impl FromStr for Op {
    type Err = Report<ParseOpError>;

    fn from_str(l: &str) -> Result<Self, Self::Err> {
        let make_error = || ParseOpError(l.to_owned());
        match l.split(' ').collect::<Vec<_>>().as_slice() {
            [dir, x] => {
                let step_count = x
                    .parse::<StepCount>()
                    .into_report()
                    .change_context(make_error())?;
                let op_kind = match *dir {
                    "U" => OpKind::Up,
                    "R" => OpKind::Right,
                    "D" => OpKind::Down,
                    "L" => OpKind::Left,
                    _ => make_error().into_err()?,
                };
                Op {
                    op_kind,
                    step_count,
                }
                .into_ok()
            }
            _ => make_error().into_err()?,
        }
    }
}

fn parse_ops(s: &str) -> ESResult<Ops, ParseOpError> {
    s.lines().map(|l| l.parse()).try_collect()
}

impl Op {
    fn step_count(&self) -> StepCount {
        self.step_count
    }

    fn set_step_count(&mut self, step_count: usize) {
        self.step_count = step_count;
    }

    fn grid_iter_direction(&self) -> GridIterDirection {
        match self.op_kind {
            OpKind::Up => GridIterDirection::Up,
            OpKind::Right => GridIterDirection::Right,
            OpKind::Down => GridIterDirection::Down,
            OpKind::Left => GridIterDirection::Left,
        }
    }

    fn move_pos(&self, pos: &mut GridPos) {
        let dir = self.grid_iter_direction();
        let adjusted_delta = dir.delta() * self.step_count() as isize;
        *pos += adjusted_delta;
    }

    fn move_pos_isize(&self, pos: &mut GridPosISize) {
        let dir = self.grid_iter_direction();
        let adjusted_delta = dir.delta() * self.step_count() as isize;
        *pos += adjusted_delta;
    }
}

type Ops = Vec<Op>;

fn split_ops_in_single_steps(ops: &Ops) -> Ops {
    ops.iter()
        .flat_map(|op| {
            (0..op.step_count()).map(|_| {
                let mut op = *op;
                op.set_step_count(1);
                op
            })
        })
        .collect()
}

struct PrettyOps<'a>(&'a Ops);

impl<'a> std::fmt::Display for PrettyOps<'a> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.0.iter().try_for_each(|e| writeln!(f, "{e}"))
    }
}

#[derive(Clone, Copy, Debug, derive_more::Display, PartialEq, Eq, Hash)]
enum Tile {
    #[display(fmt = ".")]
    Empty,
    #[display(fmt = "s")]
    Start,
    #[display(fmt = "#")]
    Visited,
    #[display(fmt = "{}", "_0")]
    Tail(usize),
    #[display(fmt = "H")]
    Head,
}

impl Tile {
    fn cmp_discriminant(&self) -> usize {
        match self {
            Tile::Empty => 0,
            Tile::Start => 1,
            Tile::Visited => 2,
            Tile::Tail(_) => 3,
            Tile::Head => 4,
        }
    }
}

impl Ord for Tile {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        match (self, other) {
            (Tile::Tail(self_tail), Tile::Tail(other_tail)) => {
                // Smaller tail index is greater than a bigger one for output purposes
                self_tail.cmp(other_tail).reverse()
            }
            _ => self.cmp_discriminant().cmp(&other.cmp_discriminant()),
        }
    }
}

impl PartialOrd for Tile {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl Default for Tile {
    fn default() -> Self {
        Tile::Empty
    }
}

#[derive(Debug, derive_more::Display, Clone)]
#[display(fmt = "{}", grid)]
struct RopeSimulation {
    knots: Vec<GridPos>,
    start: GridPos,
    grid: Grid<Tile>,
    visited: Vec<bool>,
}

impl RopeSimulation {
    fn new(start: GridPos, extents: &GridExtents, knot_count: usize) -> RopeSimulation {
        let grid = Grid::new(
            extents.row_range.end as usize,
            extents.col_range.end as usize,
        );
        let visited_len = grid.rows * grid.cols;
        let mut s = RopeSimulation {
            knots: vec![start; knot_count],
            start,
            grid,
            visited: vec![false; visited_len],
        };
        s.grid[start] = Tile::Start;
        s
    }

    fn is_knots_touching(&self, head_index: usize, tail_index: usize) -> bool {
        Direction9::iter()
            .map(|dir| self.knots[tail_index] + dir.delta())
            .any(|neighbor| neighbor == self.knots[head_index])
    }

    #[allow(unused)]
    fn reset_grid(&mut self) {
        (0..self.grid.rows).for_each(|row| {
            (0..self.grid.cols).for_each(|col| {
                self.grid[(row, col).into()] = Tile::Empty;
            });
        });
    }

    #[allow(unused)]
    fn update_grid(&mut self) {
        self.reset_grid();

        // Add visited nodes to grid, will be overidden later.
        self.visited
            .iter()
            .enumerate()
            .filter(|(_, e)| **e)
            .for_each(|(i, _)| {
                let pos = self.grid.get_pos_from_linear_index(i);
                self.grid[pos] = Tile::Visited;
            });

        // Add the knots to the grid.
        self.knots
            .iter()
            .enumerate()
            .for_each(|(knot_index, knot_pos)| {
                let prev_tile = self.grid[*knot_pos];
                let new_tile = match knot_index {
                    0 => Tile::Head,
                    _ => Tile::Tail(knot_index),
                };
                if new_tile > prev_tile {
                    self.grid[*knot_pos] = new_tile;
                }
            });
    }

    fn direction_unit_delta(from: &GridPos, to: &GridPos) -> GridPosDelta {
        let mut delta = *to - *from;
        if delta.row_delta != 0 {
            delta.row_delta /= delta.row_delta.abs();
        }
        if delta.col_delta != 0 {
            delta.col_delta /= delta.col_delta.abs();
        }
        delta
    }

    fn move_tail_towards_head(&mut self, head_index: usize, tail_index: usize) {
        let head_pos = self.knots[head_index];
        let tail_pos = self.knots[tail_index];
        let delta = Self::direction_unit_delta(&tail_pos, &head_pos);
        self.knots[tail_index] += delta;
    }

    fn process_op(&mut self, op: &Op) {
        // Move head once per op.
        op.move_pos(&mut self.knots[0]);

        // Adjust tail positions, using the current front knot as a head and the
        // next knot as the tail.
        // Iterate one less knot, because the last knot is never a head.
        (0..self.knots.len() - 1).for_each(|front_index| {
            let tail_index = front_index + 1;

            if !self.is_knots_touching(front_index, tail_index) {
                self.move_tail_towards_head(front_index, tail_index)
            }

            // If processing the last tail, mark the tile it's on as visited.
            if tail_index == self.knots.len() - 1 {
                let linearized_index = self.grid.get_element_index(self.knots[tail_index]);
                self.visited[linearized_index] = true;
            }
        });
    }

    fn simulate_step(&mut self, ops: &Ops, op_index: usize) {
        let op = &ops[op_index];
        self.process_op(op);
    }

    fn simulate(&mut self, ops: &Ops) {
        ops.iter().enumerate().for_each(|(_i, op)| {
            self.process_op(op);
        });
    }

    fn tail_visited_count(&self) -> usize {
        self.visited.iter().filter(|e| **e).count()
    }
}

fn compute_grid_extents(ops: &Ops) -> GridExtents {
    GridExtents::compute_grid_extents(ops.iter().scan(GridPosISize::default(), |pos, op| {
        op.move_pos_isize(pos);
        Some(*pos)
    }))
}

#[derive(Clone)]
struct RopeSimulationState {
    simulation: RopeSimulation,
    ops: Ops,
    current_op_index: usize,
}

fn prepare_simulation(s: &str, knot_count: usize) -> ESResult<RopeSimulationState, PuzzleError> {
    let ops = parse_ops(s).change_context(PuzzleError)?;
    let extents = compute_grid_extents(&ops);
    let normalized_extents = extents.normalized();
    let normalized_origin = extents.normalized_pos(&GridPos::default());
    let simulation = RopeSimulation::new(normalized_origin, &normalized_extents, knot_count);
    let ops = split_ops_in_single_steps(&ops);
    RopeSimulationState {
        simulation,
        ops,
        current_op_index: 0,
    }
    .into_ok()
}

fn part_compute(s: &str, knot_count: usize) -> ESResult<usize, PuzzleError> {
    let state = prepare_simulation(s, knot_count)?;
    let (mut s, ops) = (state.simulation, state.ops);
    s.simulate(&ops);
    s.tail_visited_count().into_ok()
}

fn part1_compute(s: &str) -> ESResult<usize, PuzzleError> {
    part_compute(s, 2)
}

fn part2_compute(s: &str) -> ESResult<usize, PuzzleError> {
    part_compute(s, 10)
}

pub fn part1(input: &Path) -> ESResult<(), PuzzleError> {
    let s = std::fs::read_to_string(input)
        .into_report()
        .change_context(PuzzleError)?;
    let res = part1_compute(&s)?;
    println!("p1: {}", res);
    Ok(())
}

pub fn part2(input: &Path) -> ESResult<(), PuzzleError> {
    let s = std::fs::read_to_string(input)
        .into_report()
        .change_context(PuzzleError)?;
    let res = part2_compute(&s)?;
    println!("p2: {}", res);
    Ok(())
}

struct GuiState {
    simulation_state: RopeSimulationState,
    initial_simulation_state: RopeSimulationState,
    paused: bool,
    // steps per frame update tick
    speed: usize,
    egui_context: Option<egui::Context>,
    close_requested: bool,
}

impl GuiState {
    fn try_new(input: &str, knot_count: usize) -> ESResult<Self, PuzzleError> {
        let state = prepare_simulation(input, knot_count)?;

        Self {
            simulation_state: state.clone(),
            initial_simulation_state: state,
            paused: true,
            speed: 1,
            egui_context: None,
            close_requested: false,
        }
        .into_ok()
    }
}

struct MyApp {
    state: Arc<Mutex<GuiState>>,
}

impl MyApp {
    fn try_new(input: &str, knot_count: usize) -> ESResult<Self, PuzzleError> {
        let state = GuiState::try_new(input, knot_count)?;
        let state = Arc::new(Mutex::new(state));
        Self { state }.into_ok()
    }
}

impl MyApp {
    fn show_panel(&mut self, ctx: &egui::Context) {
        egui::SidePanel::left("left").show(ctx, |ui| {
            let mut state = self.state.lock().unwrap();
            let paused = state.paused;

            if ui.button("Reset").clicked() {
                state.simulation_state.current_op_index = 0;
                state.paused = true;
                state.simulation_state = state.initial_simulation_state.clone();
            };

            ui.toggle_value(&mut state.paused, {
                if paused {
                    "▶️"
                } else {
                    "⏸️"
                }
            });

            ui.horizontal(|ui| {
                ui.label("Speeeed: ");
                ui.add(egui::Slider::new(&mut state.speed, 1..=20));
            });
        });
    }

    fn update_grid(&mut self, ui: &mut egui::Ui) {
        let state = self.state.lock().unwrap();

        for row in 0..state.simulation_state.simulation.grid.rows {
            for col in 0..state.simulation_state.simulation.grid.cols {
                let pos = (row, col).into();
                let tile = state.simulation_state.simulation.grid[pos];
                ui.label(format!("{tile}"));
            }
            ui.end_row();
        }
    }
}

impl eframe::App for MyApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        self.show_panel(ctx);

        // Show the grid model in the gui.
        egui::CentralPanel::default().show(ctx, |ui| {
            egui::Grid::new("grid").show(ui, |ui| {
                self.update_grid(ui);
            });
        });

        ctx.request_repaint_after(std::time::Duration::from_millis(1000));
    }

    fn on_close_event(&mut self) -> bool {
        self.state.lock().unwrap().close_requested = true;
        true
    }
}

fn start_simulation_thread(
    gui_state: Arc<Mutex<GuiState>>,
    barrier: Arc<Barrier>,
) -> std::thread::JoinHandle<()> {
    std::thread::spawn(move || {
        // Wait for egui context to be assigned by gui thread.
        barrier.wait();
        let egui_context = gui_state
            .lock()
            .unwrap()
            .egui_context
            .take()
            .expect("Expected egui context to exist");

        loop {
            if gui_state.lock().unwrap().close_requested {
                break;
            }
            let duration = {
                let gui_state = gui_state.lock().unwrap();
                (1000 / gui_state.speed) as u64
            };

            std::thread::sleep(std::time::Duration::from_millis(duration));

            let mut state = gui_state.lock().unwrap();
            if !state.paused {
                // Update grid model.
                state.simulation_state.simulation.update_grid();

                // Simulate next step. Make sure to request repaint one more time
                // after simulation was finished.
                if state.simulation_state.current_op_index < state.simulation_state.ops.len() {
                    if state.simulation_state.current_op_index
                        < state.simulation_state.ops.len() - 1
                    {
                        let ops_copy = state.simulation_state.ops.clone();
                        let current_op_index = state.simulation_state.current_op_index;
                        state
                            .simulation_state
                            .simulation
                            .simulate_step(&ops_copy, current_op_index);
                        state.simulation_state.current_op_index += 1;
                    }
                    egui_context.request_repaint();
                }
            }
        }
    })
}

pub fn run_gui(input: &str) -> ESResult<(), PuzzleError> {
    let options = eframe::NativeOptions {
        initial_window_size: Some(egui::vec2(1024.0, 768.0)),
        ..Default::default()
    };

    let app = MyApp::try_new(input, 2)?;

    let barrier = Arc::new(Barrier::new(2));

    let simulation_thread = start_simulation_thread(app.state.clone(), barrier.clone());

    let gui_state = app.state.clone();
    eframe::run_native(
        "Advent 2022 Day 09",
        options,
        Box::new(move |cc| {
            let mut state_guard = gui_state.lock().unwrap();
            state_guard.egui_context = Some(cc.egui_ctx.clone());
            barrier.wait();
            Box::new(app)
        }),
    );
    simulation_thread.join().unwrap();
    Ok(())
}

// #[derive(Debug, thiserror::Error)]
// pub enum Error {
//     #[error(transparent)]
//     Io(#[from] std::io::Error),
//     #[error("no solution found")]
//     NoSolution,
// }

#[derive(Debug, thiserror::Error)]
#[error("Failed to parse op: {0}")]
pub struct ParseOpError(String);

#[derive(Debug, thiserror::Error)]
#[error("Something failed")]
pub struct PuzzleError;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test() -> ESResult<(), PuzzleError> {
        let s = "R 4
U 4
L 3
D 1
R 4
D 1
L 5
R 2";
        assert_eq!(part1_compute(s)?, 13);
        assert_eq!(part2_compute(s)?, 1);

        let s = "R 5
U 8
L 8
D 3
R 17
D 10
L 25
U 20";
        assert_eq!(part2_compute(s)?, 36);
        Ok(())
    }
}
