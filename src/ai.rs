
mod bob;
mod noob;
mod john;

pub use bob::BobAI;
pub use noob::NoobAI;
pub use john::John;

use std::ops::Neg;

use rand::seq::SliceRandom;

use super::*;


const LOST: i32 = -1_000_000;
const WIN: i32 = 1_000_000;

pub struct RandomAI {}

impl Actor for RandomAI {
    fn next(&mut self, board: &Board) -> CellPos {
        random()
    }
}

fn valid_move(board: &Board, cp: CellPos) -> bool {
    if board[cp].is_some() {
        return false;
    }

    let mut has_d_neigbor = false;

    for i in 0..8 {
        if let Some(next) = cp.try_add(dir(i)) {
            has_d_neigbor |= board[next].is_some();
            if let Some(next) = next.try_add(dir(i)) {
                has_d_neigbor |= board[next].is_some();
            }
        }
    }

    has_d_neigbor
}

fn bobs_shallow_eval(board: &Board) -> i32 {
    let mut counter_f = vec![[0; 2]; 6];
    let mut counter_e = vec![[0; 2]; 6];

    // bob is still a noob he will not consider 6s bad 


    for x in 0..15 {
        for y in 0..15 {
            let cp = cell(x, y);

            let dir_count = board.compute_dir_lengths_from(cp);

            if board[cp].is_none() {
                for i in 0..4 {
                    let Some(next1) = cp.try_add(dir(i)) else {
                        continue;
                    };
                    let Some(next2) = cp.try_add(dir(i+4)) else {
                        continue;
                    };
                    if board[next1].is_none() || board[next1] != board[next2] {
                        continue;
                    }

                    let count = dir_count[i]+dir_count[i+4];

                    let mut bounded = 2;

                    if let Some(next) = cp.try_add((dir_count[i]+1) * dir(i)) {
                        if board[next].is_none() {
                            bounded -= 1;
                        } 
                    }
                    if let Some(next) = cp.try_add((dir_count[i+4]+1)*dir(i+4)) {
                        if board[next].is_none() {
                            bounded -= 1;
                        } 
                    } 

                    if bounded == 2 {
                        continue;
                    }
                    if count >= 5 {
                        continue;
                    }

                    if board[next1].unwrap() == board.turn {
                        counter_f[count][bounded] += count;
                    }
                    else {
                        counter_e[count][bounded] += count;
                    }
                }
                continue;
            }

            for i in 0..4 {
                let count = dir_count[i]+dir_count[i+4]-1;
                let mut bounded = 2;
                if let Some(next) = cp.try_add((dir_count[i])*dir(i)) {
                    if board[next].is_none() {
                        bounded -= 1;
                    } 
                }
                if let Some(next) = cp.try_add((dir_count[i+4])*dir(i+4)) {
                    if board[next].is_none() {
                        bounded -= 1;
                    } 
                }
                if bounded == 2 {
                    continue;
                }

                if count >= 6 {
                    continue;
                }
                assert_ne!(count, 0);

                if board[cp].unwrap() == board.turn {
                    counter_f[count][bounded] += 1;
                }
                else {
                    counter_e[count][bounded] += 1;
                }
            }
        }
    }

    // bob is aware that each count x is being counted x times :)
    if counter_f[5][0] + counter_f[5][1] != 0 {
        return WIN;
    }
    if counter_e[5][0]+counter_e[5][1] != 0 {
        return LOST;
    }
    if counter_f[4][0]+counter_f[4][1] != 0 {
        return WIN;
    }
    if counter_e[4][0] != 0 {
        return LOST;
    }
    if counter_f[3][0] != 0 && counter_e[4][1] == 0 {
        return WIN;
    }
    if counter_e[4][1]/4 > 0 && (counter_e[4][1]/4 + counter_e[3][0]/3) > 1 {
        return LOST;
    }
    if counter_e[3][0]/3 > 1 && (counter_f[3][0]+counter_f[3][1]) == 0  {
        return LOST;
    }

    // assumptions here: 
        // we have no fours
        // we have no unbounded 3s or opponent has a four

    let mut result = 0;
    let positives = [
        counter_f[2][0] * 10,
        counter_f[2][1] * 1,
        counter_f[3][0] * 300,
        counter_f[3][1] * 20
    ];
    let negatives = [
        counter_e[2][0] * 5,
        counter_e[2][1] * 1,
        counter_e[4][1] * 70,
        counter_e[3][0] * 70,
        counter_e[3][1] * 10
    ];

    for positive in positives {
        result += positive as i32;
    }
    for negative in negatives {
        result -= negative as i32;
    }

    result
}