/************************************************************************
 *                                                                      *
 * Copyright 2014 Urban Hafner                                          *
 * Copyright 2015 Urban Hafner, Igor Polyakov                           *
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

#![cfg(test)]
#![allow(unused_must_use)]

pub use board::Black;
pub use board::Board;
pub use board::Pass;
pub use board::Play;
pub use board::White;
pub use fixtures::load_board;
pub use ruleset::Minimal;

describe! score {

    it "counting the simple case" {
        let b = load_board("score/simple");
        let score = b.score();
        assert_eq!(8, score.black_stones);
        assert_eq!(8, score.white_stones);
        assert_eq!(White, score.color());
        assert_eq!("W+6.5", format!("{}", score));
    }

    it "counting disjoint territory" {
        let b = load_board("score/disjoint");
        let score = b.score();
        assert_eq!(9, score.black_stones);
        assert_eq!(16, score.white_stones);
        assert_eq!(White, score.color());
        assert_eq!("W+13.5", format!("{}", score));
    }

    it "counting with neutral points" {
        let b = load_board("score/dame");
        let score = b.score();
        assert_eq!(4, score.black_stones);
        assert_eq!(20, score.white_stones);
        assert_eq!(White, score.color());
        assert_eq!("W+22.5", format!("{}", score));
    }

}
