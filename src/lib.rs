#![feature(async_fn_in_trait)]

mod actor;
mod board;
mod game_manager;
mod ai;
mod textures;

use std::sync::{Arc, RwLock, mpsc};

use ellipsoid::prelude::*;

use ellipsoid::prelude::winit::event::ElementState;
use rand::{Rng, random};
use rand::distributions::{Distribution, Standard};

use actor::Actor;
use board::{Board, Stone, CellPos, cell, dir};
use game_manager::GameManager;
use textures::Txts;

struct Gomoku {
    graphics: Graphics<Txts>,
    board: Arc<RwLock<Board>>,
    game_manager_thread: std::thread::JoinHandle<()>,
    player_move_transmitter: mpsc::Sender<CellPos>,
    player_black: bool,
    mouse_pos: Vec2,
}

impl App<Txts> for Gomoku {
    async fn new(window: winit::window::Window) -> Self {
        let board = Arc::new(RwLock::new(Board::new(random::<Stone>())));

        let (player_move_transmitter, player_move_receiver) = mpsc::channel();

        let board_clone = board.clone();
        let game_manager_thread = std::thread::spawn(|| {
            let game_manager = GameManager::new(board_clone, Box::new(actor::Player::new(player_move_receiver)), Box::new(ai::BobAI { depth: 3 }));
            game_manager.run();
        });

        Self {
            board,
            game_manager_thread,
            graphics: Graphics::new(window).await,
            player_move_transmitter,
            mouse_pos: Vec2::ZERO,
            player_black: true
        }
    }

    fn graphics(&self) -> &Graphics<Txts> {
        &self.graphics
    }

    fn graphics_mut(&mut self) -> &mut Graphics<Txts> {
        &mut self.graphics
    }

    fn input(&mut self, event: &WindowEvent) -> bool {
        match event {
            WindowEvent::CursorMoved { position, .. } => {
                let window_size = self.graphics.window().inner_size();
                let window_size = vec2(window_size.width as f32, window_size.height as f32);
                let position = vec2(position.x as f32, position.y as f32) / window_size;
                self.mouse_pos = vec2(position.x, 1. - position.y);
                false
            },
            WindowEvent::MouseInput { button: winit::event::MouseButton::Left, state, ..} => {
                if !self.player_black || self.board.read().unwrap().turn != Stone::Black || state != &ElementState::Pressed {
                    return false;
                }

                let cp = self.mouse_pos * 15.;
                let cp = cell(cp.x as usize, cp.y as usize);

                self.player_move_transmitter.send(cp).unwrap();

                false
            }
            _ => false
        }
    }

    fn update(&mut self, dt: f32) {}

    fn draw(&mut self) {
        let board = self.board.read().unwrap();
        let board_gtransform = GTransform::from_translation(vec2(-1., -1.)).inflate(2.);

        let cell_mp = 1./15.;

        for x in 0..15 {
            for y in 0..15 {

                let cp = cell(x, y);

                let cell_pos = vec2(x as f32 * cell_mp, y as f32 * cell_mp);
                let cell_center = cell_pos + Vec2::ONE * cell_mp / 2.;

                let cell_gtransform = GTransform::from_translation(cell_center).inflate(cell_mp * 0.95);
                let outter_cell_gtransform = GTransform::from_translation(cell_center).inflate(cell_mp);

                let cell_shape = Shape::from_square_centered().apply(cell_gtransform).apply(board_gtransform).set_color(Color::from_hex(0x8B4513)).set_z(0.9);
                let outter_cell_shape = Shape::from_square_centered().apply(outter_cell_gtransform).apply(board_gtransform).set_color(Color::from_hex(0x808080)).set_z(0.91);

                if let Some(stone) = board[cp] {
                    let stone_shape = Shape::from_circle(20).apply(cell_gtransform.inflate(0.3)).apply(board_gtransform).set_color(match stone {
                        Stone::Black => Color::from_hex(0x000000),
                        Stone::White => Color::from_hex(0xFFFFFF)
                    });
                    self.graphics.add_geometry(stone_shape.into());
                }

                self.graphics.add_geometry(cell_shape.into());
                self.graphics.add_geometry(outter_cell_shape.into());
            }
        }
    }
}

pub async fn start() {
    ellipsoid::run::<Txts, Gomoku>().await;
}