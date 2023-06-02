use std::ops::{Neg, Mul};

use strum::Display;

use super::*;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Dir(isize, isize);

impl Neg for Dir {
    type Output = Self;

    fn neg(self) -> Self::Output {
        Self(-self.0, -self.1)
    }
}

impl Mul<usize> for Dir {
    type Output = Dir;

    fn mul(self, rhs: usize) -> Self::Output {
        Dir(self.0 * rhs as isize, self.1 * rhs as isize)
    }
}

impl Mul<Dir> for usize {
    type Output = Dir;

    fn mul(self, rhs: Dir) -> Self::Output {
        rhs * self
    }
}

// const DIRS: [[isize; 2]; 8] = [[0, 1], [1, 0], [1, 1], [1, -1], [0, -1], [-1, 0], [-1, -1], [-1, 1]];
const DIRS: [Dir; 8] = [Dir(0, 1), Dir(1, 0), Dir(1, 1), Dir(1, -1), Dir(0, -1), Dir(-1, 0), Dir(-1, -1), Dir(-1, 1)];
pub fn dir(i: usize) -> Dir {
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
    pub turn: Stone,
    pub hash: u64,
    pub cell_hashes: [[u64; 3]; 15*15]
}


#[derive(Copy, Clone, Debug, Eq, PartialEq, PartialOrd, Ord)]
pub struct CellPos(usize, usize);

impl CellPos {
    pub fn try_add(self, shift: Dir) -> Option<Self> {
        let x = self.0 as isize + shift.0;
        let y = self.1 as isize + shift.1;

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
    pub fn new() -> Self {
        let mut cell_hashes = [[0; 3]; 15*15];
        let mut hash = 0;
        for i in 0..15*15 {
            for j in 0..3 {
                cell_hashes[i][j] = random();
            }
            hash ^= cell_hashes[i][0];
        }
        Self {
            data: vec![vec![None; 15]; 15],
            turn: Stone::White,
            hash,
            cell_hashes
        }
    }
    pub fn make_move(&mut self, cp: CellPos) -> bool {
        if self[cp].is_some() {
            return false;
        }
        self.set(cp, Some(self.turn));

        self.hash ^= self.cell_hashes[cp.0*15+cp.1][0];
        self.hash ^= self.cell_hashes[cp.0*15+cp.1][self.turn as usize+1];

        self.turn = -self.turn;
        true
    }
    pub fn unmake_move(&mut self, cp: CellPos) {
        assert!(self[cp] == Some(-self.turn));

        self.turn = -self.turn;

        self.hash ^= self.cell_hashes[cp.0*15+cp.1][self.turn as usize+1];
        self.hash ^= self.cell_hashes[cp.0*15+cp.1][0];

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

        let dir_lengths = self.compute_dir_lengths_from(cp);
        
        for i in 0..4 {
            if dir_lengths[i]+dir_lengths[i+4]-1 == 5 {
                return true;
            }
        }

        false
    }
    
    pub fn compute_dir_lengths_from(&self, cp: CellPos) -> [usize; 8] {
        let mut result = [0; 8];

        for i in 0..8 {
            let dir = dir(i);
            let mut count = self[cp].is_some() as usize;
            let mut cur = cp;
            while let Some(next) = cur.try_add(dir) {
                if self[next] == self[cur] || count == 0 { 
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

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_hash() {
        let mut board = Board::new();
        board.make_move(cell(1, 0));
        board.unmake_move(cell(1, 0));
        board.make_move(cell(1, 0));
        board.make_move(cell(2, 5));

        let mut hash = 0;

        for i in 0..15*15 {
            if let Some(stone) = board[cell(i/15, i%15)] {
                hash ^= board.cell_hashes[i][stone as usize+1];
            }
            else {
                hash ^= board.cell_hashes[i][0];
            }
        }

        assert_eq!(hash, board.hash);
    }
}