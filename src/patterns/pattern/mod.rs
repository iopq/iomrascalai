/************************************************************************
 *                                                                      *
 * Copyright 2015 Urban Hafner                                          *
 *                                                                      *
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

use board::Black;
use board::Board;
use board::Coord;
use board::Empty;
use board::White;

mod test;

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Pattern {
    vec: Vec<Vec<char>>
}

impl Pattern {

    pub fn new(vec: Vec<Vec<char>>) -> Pattern {
        Pattern { vec: vec }
    }

    pub fn matches(&self, board: &Board, coord: &Coord) -> bool {
        board.neighbours8_unchecked(*coord)
            .iter()
            .all(|nc| self.matches_at(board, coord, nc))
    }

    pub fn expand(&self) -> Vec<Pattern> {
        self.rotated()
            .iter()
            .chain(self.swapped().iter())
            .cloned()
            .collect()
    }

    fn matches_at(&self, board: &Board, coord: &Coord, neighbour: &Coord) -> bool {
        let point_pattern = self.point_pattern_for(coord, neighbour);
        let is_inside = neighbour.is_inside(board.size());
        if is_inside {
            let color = board.color(neighbour);
            match point_pattern {
                'X' => { color == Black }
                'O' => { color == White }
                '?' => { true }
                'x' => { color != Black }
                'o' => { color != White }
                '.' => { color == Empty }
                ' ' => { false }
                _  => { panic!("Unknown pattern {:?}", point_pattern) }
            }
        } else {
            point_pattern == ' '
        }
    }

    fn point_pattern_for(&self, coord: &Coord, neighbour: &Coord) -> char {
        let offset_col = coord.col as isize - neighbour.col as isize;
        let offset_row = coord.row as isize - neighbour.row as isize;
        let col = (1 - offset_col) as usize;
        let row = (1 + offset_row) as usize;
        self.vec[row][col]
    }

    fn rotated(&self) -> Vec<Pattern> {
        vec!(
            self.clone(),
            self.rotated90(),
            self.rotated180(),
            self.rotated270(),
            self.horizontally_flipped(),
            self.vertically_flipped())
    }

    fn swapped(&self) -> Vec<Pattern> {
        self.rotated()
            .iter()
            .map(|pat| pat.swap())
            .collect()
    }

    fn swap(&self) -> Pattern {
        let swapped_vec = self.vec
            .iter()
            .map(|subvec|
                 subvec.iter().map(|&c| self.swap_char(c)).collect())
            .collect();
        Pattern::new(swapped_vec)
    }

    fn swap_char(&self, c: char) -> char {
        match c {
            'x' => 'o',
            'X' => 'O',
            'o' => 'x',
            'O' => 'X',
            _   => c
        }
    }

    fn rotated90(&self) -> Pattern {
        let line1 = vec!(self.vec[2][0], self.vec[1][0], self.vec[0][0]);
        let line2 = vec!(self.vec[2][1], self.vec[1][1], self.vec[0][1]);
        let line3 = vec!(self.vec[2][2], self.vec[1][2], self.vec[0][2]);
        Pattern::new(vec!(line1, line2, line3))
    }

    fn rotated180(&self) -> Pattern {
        let line1 = vec!(self.vec[2][2], self.vec[2][1], self.vec[2][0]);
        let line2 = vec!(self.vec[1][2], self.vec[1][1], self.vec[1][0]);
        let line3 = vec!(self.vec[0][2], self.vec[0][1], self.vec[0][0]);
        Pattern::new(vec!(line1, line2, line3))
    }

    fn rotated270(&self) -> Pattern {
        let line1 = vec!(self.vec[0][2], self.vec[1][2], self.vec[2][2]);
        let line2 = vec!(self.vec[0][1], self.vec[1][1], self.vec[2][1]);
        let line3 = vec!(self.vec[0][0], self.vec[1][0], self.vec[2][0]);
        Pattern::new(vec!(line1, line2, line3))
    }

    fn horizontally_flipped(&self) -> Pattern {
        let line1 = vec!(self.vec[2][0], self.vec[2][1], self.vec[2][2]);
        let line2 = vec!(self.vec[1][0], self.vec[1][1], self.vec[1][2]);
        let line3 = vec!(self.vec[0][0], self.vec[0][1], self.vec[0][2]);
        Pattern::new(vec!(line1, line2, line3))
    }

    fn vertically_flipped(&self) -> Pattern {
        let line1 = vec!(self.vec[0][2], self.vec[0][1], self.vec[0][0]);
        let line2 = vec!(self.vec[1][2], self.vec[1][1], self.vec[1][0]);
        let line3 = vec!(self.vec[2][2], self.vec[2][1], self.vec[2][0]);
        Pattern::new(vec!(line1, line2, line3))
    }
}
