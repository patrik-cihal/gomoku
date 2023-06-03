use crate::ai::{BoardState, bobs_shallow_eval};

use super::*;

pub struct GameManager {
    board: Arc<RwLock<Board>>,
    black_actor: Box<dyn Actor>,
    white_actor: Box<dyn Actor>,
}

impl GameManager {
    pub fn new(board: Arc<RwLock<Board>>, black_actor: Box<dyn Actor>, white_actor: Box<dyn Actor>) -> Self {
        Self {
            black_actor,
            white_actor,
            board,
        }
    }
    pub fn run(mut self) {
        loop {
            std::thread::sleep(std::time::Duration::from_millis(100));
            let board = self.board.read().unwrap();
            let next_move = match board.turn {
                Stone::Black => self.black_actor.next(&board),
                Stone::White => self.white_actor.next(&board),
            };
            drop(board);
            let mut board = self.board.write().unwrap();
            if board.make_move(next_move) {
                println!("TURN: {:?}", board.turn);
                println!("{:?}", BoardState::compute(&board));
                println!("{}", bobs_shallow_eval(&board, true));
                if board.check_win_from(next_move) {
                    println!("{} wins!", -board.turn);
                    break;
                }
            }
        }
    }
}