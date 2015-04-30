/************************************************************************
 *                                                                      *
 * Copyright 2014 Urban Hafner, Thomas Poinsot                          *
 * Copyright 2015 Urban Hafner, Thomas Poinsot, Igor Polyakov           *
 * This file is part of Iomrascálaí.                                    *
 *                                                                      *
 * Iomrascálaí is free software: you can redistribute it and/or modify  *
 * it under the terms of the GNU General Public License as published by *
 * the Free Software Foundation, either version 3 of the License, or    *
 * (at your option) any later version.                                  *
 *                                                                      *
 * Iomrascálaí is distributed in the hope that it will be useful,       *
 * but WITHOUT ANY WARRANTY; without even the implied warranty of       *
 * MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the        *
 * GNU General Public License for more details.                         *
 *                                                                      *
 * You should have received a copy of the GNU General Public License    *
 * along with Iomrascálaí.  If not, see <http://www.gnu.org/licenses/>. *
 *                                                                      *
 ************************************************************************/

pub use self::no_eyes::NoEyesPlayout;
pub use self::no_eyes::NoSelfAtariPlayout;
use board::Board;
use board::Color;
use board::Move;
use board::Pass;
use board::Play;
use config::Config;

use rand::{Rng, XorShiftRng};

mod no_eyes;
mod test;

pub fn factory(opt: Option<String>, config: Config) -> Box<Playout> {
    match opt.as_ref().map(::std::ops::Deref::deref) {
        Some("light") => Box::new(NoEyesPlayout),
        _             => Box::new(NoSelfAtariPlayout::new(config)),
    }
}

pub trait Playout: Sync + Send {

    fn run(&self, board: &mut Board, initial_move: Option<&Move>, rng: &mut XorShiftRng) -> PlayoutResult {
        let mut played_moves = Vec::new();

        initial_move.map(|&m| {
            board.play_legal_move(m);
            played_moves.push(m);
        });

        let max_moves = self.max_moves(board.size());
        while !board.is_game_over() && played_moves.len() < max_moves {
            let m = self.select_move(board, rng);
            board.play_legal_move(m);
            played_moves.push(m);
        }
        PlayoutResult::new(played_moves, board.winner())
    }

    fn is_playable(&self, board: &Board, m: &Move) -> bool;

    fn max_moves(&self, size: u8) -> usize {
        size as usize * size as usize * 3
    }

    fn select_move(&self, board: &Board, rng: &mut XorShiftRng) -> Move {
        let color = board.next_player();
        
        //if own group of more than one stone has one liberty, check if it can be captured
        if self.check_for_atari() {
            let mut in_danger = board.chains().iter()
                .filter(|chain| chain.color() == color && chain.coords().len() > 1 && chain.liberties().len() == 1);
   
            if let Some(chain) = in_danger.next() {
                let solutions =  if self.check_for_ladders() {
                    board.save_group(chain)
                } else {
                    board.fix_atari_no_ladder_check(chain)
                };
                
                if solutions.len() > 0 { //if we can actually save it
                    let random = rng.gen::<usize>() % solutions.len();
                    return solutions[random];
                }
            }
        }

        
        let vacant = board.vacant();
        let playable_move = vacant
            .iter()
            .map(|c| Play(color, c.col, c.row))
            .position(|m| board.is_legal(m).is_ok() && self.is_playable(board, &m));
        if let Some(first) = playable_move {
            let mut include_pass = 0;
            loop {
                let r = first + rng.gen::<usize>() % (vacant.len() - first + include_pass);
                
                if r == vacant.len() {
                    return Pass(color);
                }
                let c = vacant[r];
                let m = Play(color, c.col, c.row);
                if board.is_legal(m).is_ok() && self.is_playable(board, &m) {
                    if include_pass == 0 && !board.is_not_self_atari(&m) {
                        include_pass = 1; //try to pass in a seki sometimes
                    } else {
                        return board.play_in_middle_of_eye(m).unwrap_or(m);
                    }
                }
            }
        } else {
            Pass(color)
        }
    }

    fn playout_type(&self) -> &'static str;
    
    fn check_for_ladders(&self) -> bool;
    
    fn check_for_atari(&self) -> bool;
}

pub struct PlayoutResult {
    moves: Vec<Move>,
    winner: Color,
}

impl PlayoutResult {

    pub fn new(moves: Vec<Move>, winner: Color) -> PlayoutResult {
        PlayoutResult { moves: moves, winner: winner }
    }

    pub fn moves(&self) -> &Vec<Move> {
        &self.moves
    }

    pub fn winner(&self) -> Color {
        self.winner
    }

}
