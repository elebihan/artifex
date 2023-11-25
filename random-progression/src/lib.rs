//
// Copyright (C) 2022 Eric Le Bihan <eric.le.bihan.dev@free.fr>
//
// SPDX-License-Identifier: MIT
//

use rand::{distributions::Uniform, thread_rng, Rng};
use std::collections::VecDeque;

pub struct RandomProgression {
    positions: VecDeque<u8>,
}

impl RandomProgression {
    pub fn new() -> Self {
        let mut rng = thread_rng();
        let count = rng.gen_range(1..10);
        let mut positions: Vec<u8> = (&mut rng)
            .sample_iter(Uniform::new(1, 99))
            .take(count)
            .collect();
        positions.push(100);
        positions.sort();

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
            println!("Progression: {}%", position);
        }
    }
}
