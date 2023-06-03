use std::collections::HashMap;

use super::*;

#[derive(Eq, PartialEq, Hash)]
struct MemoryEntry(u8, u64);
pub struct BobAI {
    pub depth: u8,
    memory: HashMap<MemoryEntry, (i32, Option<CellPos>)>,
    pub used_memory: usize,
    pub computed_positions: usize
}

impl BobAI {
    pub fn new(depth: u8) -> Self {
        BobAI { depth, memory: HashMap::default(), used_memory: 0, computed_positions: 0 }
    }
    
   
    pub fn minmax(&mut self, cur_depth: u8, board: &mut Board, mut alpha: i32, beta: i32) -> (i32, Option<CellPos>) {
        // things we can assume here: 
            // 1. we haven't won already
        if let Some(result) = self.memory.get(&MemoryEntry(cur_depth, board.hash)) {
            self.used_memory += 1;
            return *result;
        }
        self.computed_positions += 1;
        if cur_depth == self.depth {
            let result = (bobs_shallow_eval(board, false), None);
            self.memory.insert(MemoryEntry(cur_depth, board.hash), result);
            return result;
        }
        let mut result = (LOST-1, None);
        let mut moves = vec![];
        for x in 0..15 {
            for y in 0..15 {
                let cp = cell(x, y);

                if !valid_move(board, cp) {
                    continue;
                }

                moves.push(cp);
            }
        }

        let mut moves = moves.into_iter().map(|cp| {
            board.make_move(cp);
            let result = (bobs_shallow_eval(board, false), cp);
            board.unmake_move(cp);
            result
        }).collect::<Vec<_>>();
        moves.sort_unstable_by_key(|x| x.0);

        // for new_depth in cur_depth+2..self.depth {
        //     let mut local_alpha = alpha;
        //     moves = moves.into_iter().map(|(_leval, cp)| {

        //         board.make_move(cp);
        //         let result = (-self.minmax(new_depth, board, -beta, -local_alpha).0, cp);
        //         if result.0 > local_alpha {
        //             local_alpha = result.0;
        //         }
        //         board.unmake_move(cp);
        //         result
        //     }).collect::<Vec<_>>();
        //     moves.sort_by(|a, b| b.0.partial_cmp(&a.0).unwrap());
        // }
        for (_, cp) in moves {
            assert!(board.make_move(cp));

            if board.check_win_from(cp) {
                board.unmake_move(cp);
                return (WIN, Some(cp));
            }

            let mut eval = self.minmax(cur_depth+1, board, -beta, -alpha);
            eval.0 = -eval.0;

            board.unmake_move(cp);

            if eval.0 > result.0 {
                result = (eval.0, Some(cp));
                if eval.0 > alpha {
                    alpha = eval.0;
                    if alpha >= beta {
                        return result;
                    }
                }
            }
        }
        // self.memory.insert(MemoryEntry(cur_depth, board.hash), result);
        result

    }
}

impl Actor for BobAI {
    fn next(&mut self, board: &Board) -> CellPos {
        if board.free_positions().count() == 15*15 {
            return cell(7, 7);
        }
        let mut board = board.clone();
        let last_memory_count = self.used_memory;
        let last_computed_positions_count = self.computed_positions;
        let next_move = self.minmax(0, &mut board, LOST-1, WIN+1);
        println!("---------BOB EVAL------------");
        println!("MEMORY USE: {}", self.used_memory-last_memory_count);
        println!("POSITIONS COUNT: {}", self.computed_positions-last_computed_positions_count);
        println!("MOVE SCORE: {}", next_move.0);
        next_move.1.unwrap()
    }
}