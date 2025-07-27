//
// Copyright (C) 2022 Eric Le Bihan <eric.le.bihan.dev@free.fr>
//
// SPDX-License-Identifier: MIT
//

use rand::{
    distr::{Distribution, Uniform},
    Rng,
};
use std::collections::VecDeque;

pub struct RandomProgression {
    positions: VecDeque<u8>,
}

impl RandomProgression {
    /// Create a new random progression between 1 and 99.
    ///
    /// # Panics
    /// Panics if internally, a uniform range from 1 to 99 can not be created.
    #[must_use]
    pub fn new() -> Self {
        let mut rng = rand::rng();
        let count = rng.random_range(1..10);
        let positions_range = Uniform::new_inclusive(1, 99).expect("Uniform range should be valid");
        let mut positions: Vec<u8> = positions_range.sample_iter(&mut rng).take(count).collect();
        positions.push(100);
        positions.sort_unstable();

        Self {
            positions: VecDeque::from(positions),
        }
    }
}

impl Default for RandomProgression {
    fn default() -> Self {
        Self::new()
    }
}

impl std::iter::Iterator for RandomProgression {
    type Item = u8;

    fn next(&mut self) -> Option<Self::Item> {
        self.positions.pop_front()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn basic() {
        let progression = RandomProgression::new();
        for position in progression {
            println!("Progression: {position}%");
        }
    }
}
