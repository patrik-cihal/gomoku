use std::ops::Neg;

use strum::Display;

use super::*;

const DIRS: [[isize; 2]; 8] = [[0, 1], [1, 0], [1, 1], [1, -1], [0, -1], [-1, 0], [-1, -1], [-1, 1]];
pub fn dir(i: usize) -> [isize; 2] {
    DIRS[i%8]
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Display)]
pub enum Stone {
    Black,
    White
}

impl Neg for Stone {
    type Output = Self;

    fn neg(self) -> Self::Output {
        match self {
            Stone::Black => Stone::White,
            Stone::White => Stone::Black
        }
    }
}

impl Distribution<Stone> for Standard {
    fn sample<R: Rng + ?Sized>(&self, rng: &mut R) -> Stone {
        match rng.gen_range(0..2) {
            0 => Stone::Black,
            _ => Stone::White
        }
    }
}

#[derive(Clone, Debug)]
pub struct Board {
    data: Vec<Vec<Option<Stone>>>,
    pub turn: Stone
}


#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub struct CellPos(usize, usize);

impl CellPos {
    pub fn try_add(self, shift: [isize; 2]) -> Option<Self> {
        let x = self.0 as isize + shift[0];
        let y = self.1 as isize + shift[1];

        if x < 0 || x >= 15 || y < 0 || y >= 15 {
            None
        } else {
            Some(cell(x as usize, y as usize))
        }
    }
}

pub fn cell(x: usize, y: usize) -> CellPos {
    assert!(x < 15 && y < 15);
    CellPos(x, y)
}

impl Distribution<CellPos> for Standard {
    fn sample<R: Rng + ?Sized>(&self, rng: &mut R) -> CellPos {
        CellPos(rng.gen_range(0..15), rng.gen_range(0..15))
    }
}

impl std::ops::Index<CellPos> for Board {
    type Output = Option<Stone>;

    fn index(&self, index: CellPos) -> &Self::Output {
        &self.data[index.0][index.1]
    }
}

impl Board {
    pub fn new(turn: Stone) -> Self {
        Self {
            data: vec![vec![None; 15]; 15],
            turn
        }
    }
    pub fn make_move(&mut self, cp: CellPos) -> bool {
        if self[cp].is_some() {
            return false;
        }
        self.set(cp, Some(self.turn));
        self.turn = -self.turn;
        true
    }
    pub fn unmake_move(&mut self, cp: CellPos) {
        assert!(self[cp].is_some());
        self.turn = -self.turn;
        self.set(cp, None);
    }

    fn set(&mut self, cp: CellPos, stone: Option<Stone>) {
        self.data[cp.0][cp.1] = stone;
    }
    pub fn free_positions(&self) -> FreePosIterator {
        FreePosIterator {
            board: self,
            i: 0
        }
    }

    pub fn check_win(&self) -> bool {
        for x in 0..15 {
            for y in 0..15 {
                if self.check_win_from(cell(x, y)) {
                    return true;
                }
            }
        }
        false
    }
    pub fn check_win_from(&self, cp: CellPos) -> bool {
        if self[cp].is_none() {
            return false;
        }

        for (i, l) in self.compute_dir_lengths_from(cp).into_iter().enumerate() {
            if l==5 {
                if let Some(next) = cp.try_add(dir(i+4)) {
                    if self[next] == self[cp] {
                        continue;
                    }
                }
                return true;
            }
        }

        false
    }
    
    pub fn compute_dir_lengths_from(&self, cp: CellPos) -> [usize; 8] {
        let mut result = [0; 8];

        for i in 0..8 {
            let dir = dir(i);
            let mut count = 1;
            let mut cur = cp;
            while let Some(next) = cur.try_add(dir) {
                if self[next] == self[cp] {
                    count += 1;
                    cur = next;
                    if count == 6 {
                        break;
                    }
                } else {
                    break;
                }
            }
            result[i] = count;
        }
        result
    }
}

pub struct FreePosIterator<'a> {
    board: &'a Board,
    i: usize,
}

impl Iterator for FreePosIterator<'_> {
    type Item = CellPos;

    fn next(&mut self) -> Option<Self::Item> {
        while self.i < 15 * 15 {
            let x = self.i % 15;
            let y = self.i / 15;
            self.i += 1;
            if self.board[cell(x, y)].is_none() {
                return Some(cell(x, y));
            }
        }
        None
    }
}