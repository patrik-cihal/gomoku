use std::ops::Neg;

use rand::seq::SliceRandom;

use super::*;

pub struct RandomAI {}

impl Actor for RandomAI {
    fn next(&mut self, board: &Board) -> CellPos {
        random()
    }
}

pub struct NoobAI {
    pub depth: usize
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum NoobEval {
    Win,
    Lose,
    Uncertain
}

impl NoobAI {
    fn minmax(&self, cur_depth: usize, board: &mut Board) -> Option<(CellPos, NoobEval)> {
        if cur_depth == self.depth {
            return None;
        }
        let mut eval = None;
        for x in 0..15 {
            for y in 0..15 {
                let cp = cell(x, y);

                let mut has_neigbor = false;

                for i in 0..8 {
                    if let Some(next) = cp.try_add(dir(i)) {
                        has_neigbor |= board[next].is_some();
                    }
                }

                if !has_neigbor || !board.make_move(cp) {
                    continue;
                }
                let win = board.check_win_from(cp);

                if win {
                    board.unmake_move(cp);
                    return Some((cp, NoobEval::Win));
                }    

                let result = self.minmax(cur_depth+1, board);
                board.unmake_move(cp);

                if let Some((_, mv_eval)) = result {
                    match mv_eval {
                        NoobEval::Win => {
                            if eval.is_none() {
                                eval = Some((cp, NoobEval::Lose));
                            }
                        },
                        NoobEval::Lose => {
                            return Some((cp, NoobEval::Win));
                        },
                        NoobEval::Uncertain => {
                            eval = Some((cp, NoobEval::Uncertain));
                        }
                    }
                }
                else {
                    eval = Some((cp, NoobEval::Uncertain));
                }
            }
        }
        eval
    }
}

impl Actor for NoobAI {
    fn next(&mut self, board: &Board) -> CellPos {
        if board.free_positions().count() == 15*15 {
            return cell(7, 7);
        }
        let mut board = board.clone();
        match self.minmax(0, &mut board) {
            Some((cp, _)) => cp,
            None => board.free_positions().next().unwrap()
        }
    }

}


pub struct BobAI {
    pub depth: usize 
}

impl BobAI {
    fn valid_move(&self, cp: CellPos, board: &Board) -> bool {
        if board[cp].is_some() {
            return false;
        }

        let mut has_d_neigbor = false;

        for i in 0..8 {
            if let Some(next) = cp.try_add(dir(i)) {
                has_d_neigbor |= board[next].is_some();
                if let Some(next) = cp.try_add(dir(i)) {
                    has_d_neigbor |= board[next].is_some();
                }
            }
        }

        has_d_neigbor
    }
    fn shallow_eval(&self, board: &Board) -> f32 {
        let mut counter_f = vec![[0; 2]; 5];
        let mut counter_e = vec![[0; 2]; 5];

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

                        if let Some(next) = cp.try_add([(dir_count[i]+1) as isize*dir(i)[0], (dir_count[i]+1) as isize*dir(i)[1]]) {
                            if board[next].is_none() {
                                bounded -= 1;
                            } 
                        }
                        if let Some(next) = cp.try_add([(dir_count[i+4]+1) as isize*dir(i+4)[0], (dir_count[i+4]+1) as isize*dir(i+4)[1]]) {
                            if board[next].is_none() {
                                bounded -= 1;
                            } 
                        } 

                        if bounded == 2 {
                            continue;
                        }

                        if board[cp].unwrap() == board.turn {
                            counter_f[count][bounded] += 1;
                        }
                        else {
                            counter_e[count][bounded] += 1;
                        }
                    }
                    continue;
                }

                for i in 0..4 {
                    let count = dir_count[i]+dir_count[i+4]-1;
                    let mut bounded = 2;
                    if let Some(next) = cp.try_add([dir_count[i] as isize*dir(i)[0], dir_count[i] as isize*dir(i)[1]]) {
                        if board[next].is_none() {
                            bounded -= 1;
                        } 
                    }
                    if let Some(next) = cp.try_add([dir_count[i+4] as isize*dir(i+4)[0], dir_count[i+4] as isize*dir(i+4)[1]]) {
                        if board[next].is_none() {
                            bounded -= 1;
                        } 
                    }
                    if bounded == 2 {
                        continue;
                    }

                    if count == 6 {
                        continue;
                    }
                    assert_ne!(count, 5);
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

        if counter_f[4][0]+counter_f[4][1] != 0 {
            return 1.;
        }
        if counter_e[4][0] != 0 {
            return -1.;
        }
        if counter_f[3][0] != 0 && counter_e[4][1] == 0 {
            return 1.;
        }
        if counter_e[4][1]/4 > 1 {
            return -1.;
        }
        if counter_e[3][0]/3 > 1 && (counter_f[3][0]+counter_f[3][1]) == 0  {
            return -1.;
        }

        // assumptions here: 
            // we have no fours
            // we have no unbounded 3s or opponent has a four

        let mut result = 0.;
        result += counter_f[2][0] as f32 * 0.01;
        result += counter_f[2][1] as f32 * 0.001;
        result += counter_f[3][0] as f32 * 0.05;
        result += counter_f[3][1] as f32 * 0.02;
        result -= counter_e[2][0] as f32 * 0.005;
        result -= counter_e[2][1] as f32 * 0.001;
        result -= counter_e[4][1] as f32 * 0.07;
        result -= counter_e[3][0] as f32 * 0.03;
        result -= counter_e[3][1] as f32 * 0.01;

        result
    }
    pub fn minmax(&self, cur_depth: usize, board: &mut Board) -> (f32, Option<CellPos>) {
        // things we can assume here: 
            // 1. we haven't won already
        if cur_depth == self.depth {
            return (self.shallow_eval(board), None);
        }
        let mut result = (f32::MIN, None);
        for x in 0..15 {
            for y in 0..15 {
                let cp = cell(x, y);

                if !self.valid_move(cp, board) {
                    continue;
                }
                assert!(board.make_move(cp));

                if board.check_win_from(cp) {
                    board.unmake_move(cp);
                    return (1., Some(cp));
                }

                let eval = self.minmax(cur_depth+1, board);
                board.unmake_move(cp);

                if -eval.0 > result.0 {
                    result = (-eval.0, Some(cp));
                }
            }
        }
        result

    }
}

impl Actor for BobAI {
    fn next(&mut self, board: &Board) -> CellPos {
        if board.free_positions().count() == 15*15 {
            return cell(7, 7);
        }
        let mut board = board.clone();
        self.minmax(0, &mut board).1.unwrap()
    }
}