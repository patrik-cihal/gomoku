use std::sync::mpsc;

use super::*;

pub trait Actor {
    fn next(&mut self, board: &Board) -> CellPos;
}

pub struct Player {
    thread_receiver: mpsc::Receiver<CellPos>
}

impl Player {
    pub fn new(thread_receiver: mpsc::Receiver<CellPos>) -> Self {
        Self {
            thread_receiver
        }
    }
}

impl Actor for Player {
    fn next(&mut self, _: &Board) -> CellPos {
        self.thread_receiver.recv().unwrap()
    }
}