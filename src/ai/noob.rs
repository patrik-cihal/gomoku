use super::*;

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

