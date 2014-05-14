/************************************************************************
 *                                                                      *
 * Copyright 2014 Urban Hafner, Thomas Poinsot                          *
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

#[cfg(test)]

use board::{Board, Empty, White};

#[test]
fn test_getting_a_valid_coord_returns_a_color(){
  let b = Board::new(19, 6.5);

  assert!(b.get(1,1).unwrap()   == Empty);
  assert!(b.get(10,10).unwrap() == Empty);
}

#[test]
fn test_getting_invalid_coordinates_returns_None() {
  let b = Board::new(19, 6.5);
  assert!(b.get(14,21) == None);
  assert!(b.get(21,14) == None);
}

#[test]
fn test_19_19_is_a_valid_coordinate(){
  let b = Board::new(19, 6.5);

  assert!(b.get(19,19).unwrap() == Empty);
}

#[test]
fn test_0_0_is_not_a_valid_coordinate(){
  let b = Board::new(19, 6.5);

  assert!(b.get(0,0) == None);
}

#[test]
fn test_get_komi(){
  let b = Board::new(19, 6.5);

  assert!(b.komi() == 6.5f32)
}

#[test]
fn test_play(){
  let mut b = Board::new(19, 6.5);

  b = b.play(White, 14, 14);
  assert!(b.get(14,14).unwrap() == White);
}