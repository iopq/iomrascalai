/************************************************************************
 *                                                                      *
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

use board::Black;
use board::Pass;
use board::Play;
use board::White;
use game::Game;
use ruleset::KgsChinese;
use sgf::Parser;
use super::Node;

use std::path::Path;

#[test]
fn root_expands_the_children() {
    let game = Game::new(2, 0.5, KgsChinese);
    let root = Node::root(&game, Black);
    assert_eq!(4, root.children.len());
}

#[test]
fn expand_doesnt_add_children_to_terminal_nodes() {
    let mut game = Game::new(5, 6.5, KgsChinese);
    game = game.play(Pass(Black)).unwrap();
    game = game.play(Pass(White)).unwrap();
    let mut node = Node::new(Pass(Black));
    node.expand(&game.board(), 1);
    assert_eq!(0, node.children.len());
}

#[test]
fn expand_doesnt_add_children_if_threshold_not_met() {
    let game = Game::new(2, 0.5, KgsChinese);
    let mut node = Node::new(Pass(Black));
    node.expand(&game.board(), 2);
    assert_eq!(0, node.children.len());
}

#[test]
fn expand_adds_children_if_threshold_is_met() {
    let game = Game::new(2, 0.5, KgsChinese);
    let mut node = Node::new(Pass(Black));
    node.plays = 2;
    node.expand(&game.board(), 2);
    assert_eq!(4, node.children.len());
}

#[test]
fn unvisited_children_are_explored_first() {
    let game = Game::new(2, 0.5, KgsChinese);
    let mut root = Node::root(&game, Black);
    for _ in 0..4 {
        root.find_leaf_and_expand(&game, 1, false);
    }
    assert_eq!(4, root.children.len());
    assert!(root.children.iter().all(|n| n.plays == 1));
}

#[test]
fn find_leaf_and_expand_expands_the_leaves() {
    let game = Game::new(2, 0.5, KgsChinese);
    let mut root = Node::root(&game, Black);
    for _ in 0..4 {
        root.find_leaf_and_expand(&game, 1, false);
    }
    assert_eq!(4, root.children.len());
    assert!(root.children.iter().all(|n| n.children.len() == 3));
}

#[test]
fn find_leaf_and_expand_sets_play_on_the_root() {
    let game = Game::new(2, 0.5, KgsChinese);
    let mut root = Node::root(&game, Black);
    root.find_leaf_and_expand(&game, 1, false);
    assert_eq!(2, root.plays);
}

#[test]
fn find_leaf_and_expand_returns_the_number_of_nodes_added() {
    let game = Game::new(2, 0.5, KgsChinese);
    let mut root = Node::root(&game, Black);
    let (_,_,_,count) = root.find_leaf_and_expand(&game, 1, false);
    assert_eq!(3, count);
}

#[test]
fn the_root_needs_to_be_initialized_with_1_plays_for_correct_uct_calculations() {
    let game = Game::new(2, 0.5, KgsChinese);
    let root = Node::root(&game, Black);
    assert_eq!(1, root.plays);
    assert_eq!(1, root.wins);
 }

#[test]
fn no_super_ko_violations_in_the_children_of_the_root() {
    let parser = Parser::from_path(Path::new("fixtures/sgf/positional-superko.sgf")).unwrap();
    let game = parser.game().unwrap();
    let root = Node::root(&game, White);
    // Play(White, 2, 9) is a super ko violation
    assert!(root.children.iter().all(|n| n.m() != Play(White, 2, 9)));
}

#[test]
fn record_on_path_only_records_wins_for_the_correct_color() {
    let grandchild = Node::new(Pass(Black));
    let mut child = Node::new(Pass(White));
    child.children = vec!(grandchild);
    let mut root = Node::new(Pass(Black));
    root.children = vec!(child);

    root.record_on_path(&vec!(0, 0), Black, 0);
    assert_eq!(1, root.wins);
    assert_eq!(0, root.children[0].wins);
    assert_eq!(1, root.children[0].children[0].wins);

    root.record_on_path(&vec!(0, 0), White, 0);
    assert_eq!(1, root.wins);
    assert_eq!(1, root.children[0].wins);
    assert_eq!(1, root.children[0].children[0].wins);
}

#[test]
fn record_on_path_updates_the_descendant_counts() {
    let mut grandchild = Node::new(Pass(Black));
    // The leaf already has the correct value set
    grandchild.descendants = 5;
    let mut child = Node::new(Pass(White));
    child.children = vec!(grandchild);
    child.descendants = 1;
    let mut root = Node::new(Pass(Black));
    root.children = vec!(child);
    root.descendants = 2;

    root.record_on_path(&vec!(0, 0), Black, 5);
    assert_eq!(7, root.descendants);
    assert_eq!(6, root.children[0].descendants);
    assert_eq!(5, root.children[0].children[0].descendants);
}

#[test]
fn find_child_returns_the_correct_child() {
    let mut root = Node::new(Pass(Black));
    let child = Node::new(Play(White, 1, 1));
    root.children = vec!(Node::new(Play(Black, 5, 5)), child.clone(), Node::new(Play(Black, 3, 7)));
    assert_eq!(child, root.find_child(Play(White, 1, 1)));
}

#[test]
fn new_sets_the_descendats_to_zero() {
    let node = Node::new(Pass(Black));
    assert_eq!(0, node.descendants);
}

#[test]
fn expand_root_sets_the_correct_descendant_count_on_the_root() {
    let game = Game::new(5, 6.5, KgsChinese);
    let mut root = Node::new(Pass(Black));
    root.expand_root(&game);
    assert_eq!(25, root.descendants);
}

#[test]
fn expand_sets_the_descendant_count_if_the_node_was_expanded() {
    let game = Game::new(5, 6.5, KgsChinese);
    let board = game.board();
    let mut node = Node::new(Pass(Black));
    node.expand(&board, 0);
    assert_eq!(25, node.descendants);
}


// 2. Make sure that terminal nodes are "played", i.e. either a win or
//    a loss is reported and the wins are recorded in the tree.
// 3. Check if siblings of a terminal node will ever be explored
//    (check the uct value of a terminal node)
// 4. Maybe use the root node's plays and wins to keep track of the
//    number of playouts and the average win rate.
// 7. Test the resigning support
// 8. Make sure everything works fine in the game tree when there are
//    no moves to simulate anymore.
// 9. Implement multi threading
