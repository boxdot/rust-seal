use std::f64::{INFINITY, MAX};
use std::ops::RangeInclusive;

use pair::cursor::Cursor;
use pair::step_mask::StepMask;
use pair::strategy::Strategy as StrategyTrait;
use pair::{
    matrix::{Matrix, MatrixCell},
    alignment_set::AlignmentSet,
};

pub enum Kernel {
    Linear,
    Quadratic,
}

#[derive(Clone, Debug)]
pub struct Strategy {
    equal: f64,
    unequal: f64,
    threshold: f64,
    bounds: RangeInclusive<f64>,
}

impl Strategy {
    pub fn new(equal: f64, unequal: f64, threshold: f64, bounds: RangeInclusive<f64>) -> Self {
        Self {
            equal,
            unequal,
            threshold,
            bounds,
        }
    }

    // Dynamic time warping algorithm calculating optimal global alignment:
    pub fn dynamic_time_warping() -> Self {
        Self::new(1.0, 1.0, 0.0, 0.0..=MAX)
    }

    fn total_score(&self, score: f64) -> f64 {
        if score < *self.bounds.start() {
            self.bounds.start().clone()
        } else if score > *self.bounds.end() {
            self.bounds.end().clone()
        } else {
            score
        }
    }

    fn calculate_cell(
        &self,
        lhs: &f64,
        rhs: &f64,
        previous_scores: [f64; 3],
    ) -> MatrixCell<f64> {
        let [insert, align, delete] = previous_scores;
        let mut cell = MatrixCell::from_scores(align, delete, insert);
        let squared_distance = (lhs - rhs) * (lhs - rhs);
        let distance = squared_distance.sqrt();
        let cost = distance - self.threshold;
        let score = if cost >= 0.0 {
            (self.equal.clone() * cost) + cell.score.clone()
        } else {
            (self.unequal.clone() * cost) + cell.score.clone()
        };
        cell.score = self.total_score(score);

        cell
    }
}

impl StrategyTrait<f64> for Strategy {
    type Score = f64;

    fn alignment_set<M, E>(&self, x: &[f64], y: &[f64]) -> Result<AlignmentSet<f64, M>, E>
    where
        M: Matrix<Score = Self::Score, Error = E>,
        M: ::std::fmt::Debug,
    {
        let width = x.len() + 1;
        let height = y.len() + 1;

        let mut matrix = M::new(width, height)?;

        for y in 1..height {
            let score = INFINITY;
            let mask = StepMask::INSERT;
            *matrix.cell_mut(&Cursor::new(0, y)) = MatrixCell::new(score, mask);
        }
        for x in 1..width {
            let score = INFINITY;
            let mask = StepMask::DELETE;
            *matrix.cell_mut(&Cursor::new(x, 0)) = MatrixCell::new(score, mask);
        }
        let mut row: Vec<Self::Score> = (0..width).map(|_| INFINITY).collect();

        let mut score: f64 = MAX;
        let mut cursor = Cursor::new(1, 1);

        for (y_idx, y_val) in y.iter().enumerate() {
            let mut last_diagonal = self.total_score(if y_idx == 0 {
                0.0
            } else {
                matrix.cell(&Cursor::new(0, y_idx)).score
            });
            for (x_idx, x_val) in x.iter().enumerate() {
                let previous = [row[x_idx + 1], last_diagonal, row[x_idx]];
                let cell = self.calculate_cell(&x_val, &y_val, previous);
                let current_cursor = Cursor {
                    x: x_idx + 1,
                    y: y_idx + 1,
                };
                let current_score = cell.score.clone();
                if current_score < score {
                    score = current_score;
                    cursor = current_cursor;
                }
                *matrix.cell_mut(&current_cursor) = cell;
                let old_diagonal = row[x_idx + 1];
                row[x_idx + 1] = current_score;
                last_diagonal = old_diagonal;
            }
        }

        Ok(AlignmentSet::new(matrix, score, cursor))
    }
}