use std::collections::HashMap;

use super::*;

#[derive(Eq, PartialEq, Hash)]
struct MemoryEntry(u8, u64);

pub struct John {
    memory: HashMap<MemoryEntry, (i32, Option<CellPos>)>,
    compute: f32
}

#[derive(Debug)]
enum BoardState {
    Boring,
    // DefenseOrCounterAttack(Vec<CellPos>), this one is hell of a pain to evaluate let's not do it
    TwoMoveWin(CellPos),
    ForcedDefense(Vec<CellPos>),
    OneMoveLoss(CellPos),
    OneMoveWin(CellPos),
}

impl BoardState {
    fn priority(&self) -> u8 {
        match &self {
            Self::Boring => 0,
            // Self::DefenseOrCounterAttack(_) => 1,
            Self::TwoMoveWin(_) => 2,
            Self::ForcedDefense(_) => 3,
            Self::OneMoveLoss(_) => 4,
            Self::OneMoveWin(_) => 5,
        }
    }
}


impl BoardState {
    fn compute(board: &Board) -> Self {
        let mut result = BoardState::Boring;
        let positions = board.free_positions().filter(|cp| valid_move(board, *cp)).collect::<Vec<_>>();

        let mut update_result = |board_state: BoardState| {
            if board_state.priority() > result.priority() {
                result = board_state;
            }
            else if board_state.priority() == result.priority() {
                match board_state {
                    BoardState::Boring => panic!("You shouldn't set me (no point)."),
                    // BoardState::DefenseOrCounterAttack(mvs) => {
                    //     let BoardState::DefenseOrCounterAttack(result_mvs) = &mut result else {
                    //         panic!();
                    //     };
                    //     result_mvs.extend(mvs.into_iter());
                    // },
                    BoardState::ForcedDefense(mvs) => {
                        let BoardState::ForcedDefense(result_mvs) = &mut result else {
                            panic!();
                        };

                        result_mvs.extend(mvs.into_iter());
                    },
                    _ => return
                };
            }
            
        };

        'outer: for cp in positions {
            let dir_count = board.compute_dir_lengths_from(cp);

            for i in 0..8 { // this will evaluate splits twice, idc tho
                let Some(next) = cp.try_add(dir(i)) else {
                    continue;
                };
                let Some(stone) = board[next] else {
                    continue;
                };

                let mut back = 0;
                let front = dir_count[i];

                if let Some(prev) = cp.try_add(-dir(i)) {
                    if let Some(prev_stone) = board[prev] {
                        if prev_stone == stone {
                            back = dir_count[(i+4)%8];
                        }
                    }
                }

                // ------------ one move win -------------
                if front+back == 4 && stone == board.turn {
                    update_result(BoardState::OneMoveWin(cp));
                    break 'outer;
                }

                let mut front_bounded = true;
                let mut back_bounded = true;

                if let Some(next2) = cp.try_add((front+1)*dir(i)) {
                    front_bounded = board[next2].is_some();
                }
                if let Some(prev2) = cp.try_add((back+1)*(-dir(i))) {
                    back_bounded = board[prev2].is_some();
                }


                // ---------- one move loss ------------
                if stone != board.turn && front == 4 && back == 0 && !front_bounded {
                    update_result(BoardState::OneMoveLoss(cp));
                    continue;
                }

                // ----------- forced defense -------------

                if stone != board.turn && front+back == 4 {
                    update_result(BoardState::ForcedDefense(vec![cp]));
                }
                
                // ----------- two move win ------------

                if stone == board.turn && front == 3 && back == 0 && !back_bounded && !front_bounded {
                    update_result(BoardState::TwoMoveWin(cp));
                }

                if stone == board.turn && front == 2 && back == 1 && !back_bounded && !front_bounded {
                    update_result(BoardState::TwoMoveWin(cp));
                }


                // ----------- defense or counter attack ------------

                // if stone != board.turn && front+back == 3 && !front_bounded && !back_bounded {
                //     let Some(next2) = cp.try_add((front+1)*dir(i)) else {
                //         panic!("Must be some because it is not front_bounded");
                //     };
                //     let Some(prev) = cp.try_add((back+1)*(-dir(i))) else {
                //         panic!("Must exist because it is not back_bounded");
                //     };
                //     assert!(board[prev].is_none());
                //     assert!(board[next2].is_none());

                //     update_result(BoardState::DefenseOrCounterAttack(vec![next2, prev, cp]));
                // }

                // if stone == board.turn && front == 3 && front_bounded && !back_bounded {
                //     let Some(prev) = cp.try_add(-dir(i)) else {
                //         panic!("Must exist because not backbounded");
                //     };
                //     assert!(board[prev].is_none());
                    
                //     update_result(BoardState::DefenseOrCounterAttack(vec![cp, prev]));
                // }

                // if stone == board.turn && front + back == 3
            }
        }

        result
    }
}



impl John {
    pub fn new(compute: f32) -> Self {
        Self {
            memory: Default::default(),
            compute
        }
    }

    pub fn minimax(&mut self, board: &mut Board, mut alpha: i32, beta: i32, comp_rem: f32) -> (i32, Option<CellPos>) {
        if comp_rem < 1. {
            return (bobs_shallow_eval(board), None);
        }


        let moves_to_explore;

        match BoardState::compute(board) {
            BoardState::OneMoveWin(cp) => return (WIN, Some(cp)),
            BoardState::TwoMoveWin(cp) => return (WIN, Some(cp)),
            BoardState::OneMoveLoss(cp) => return (LOST, Some(cp)),
            BoardState::ForcedDefense(mut mvs) => {
                mvs.sort();
                mvs.dedup();
                assert!(!mvs.is_empty());
                if mvs.len() > 1 {
                    return (LOST, Some(mvs[0]));
                }
                moves_to_explore = mvs
            },
            BoardState::Boring => {
                moves_to_explore = board.free_positions().filter(|cp| valid_move(board, *cp)).collect::<Vec<_>>()
            },
        }

        let mut moves_to_explore = moves_to_explore.into_iter().map(|cp| {
            board.make_move(cp);
            let result = (bobs_shallow_eval(board), cp);
            board.unmake_move(cp);
            result
        }).collect::<Vec<_>>();
        moves_to_explore.sort();

        let mut best_move = Some(moves_to_explore[0].1);

        let new_comp_rem = comp_rem/moves_to_explore.len() as f32;
        for (_, cp) in moves_to_explore {
            board.make_move(cp);
            if board.check_win_from(cp) {
                return (WIN, Some(cp));
            }

            let mut eval = self.minimax(board, -beta, -alpha, new_comp_rem);
            eval.0 = -eval.0;
            board.unmake_move(cp);

            if eval.0 > alpha {
                alpha = eval.0;
                best_move = Some(cp);
            }
        }

        (alpha, best_move)
    }
}

impl Actor for John {
    fn next(&mut self, board: &Board) -> CellPos {
        let mut board = board.clone();
        if board.free_positions().count() == 15*15 {
            return cell(7, 7);
        }

        let result = self.minimax(&mut board, i32::MIN, i32::MAX, self.compute);

        println!("{:?}", result);
        println!("{:?}", BoardState::compute(&board));

        result.1.unwrap()
    }
}