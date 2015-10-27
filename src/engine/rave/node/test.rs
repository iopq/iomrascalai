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
use config::Config;
use game::Game;
use patterns::Matcher;
use playout::Playout;
use ruleset::KgsChinese;
use sgf::Parser;
use super::Node;

use rand::weak_rng;
use std::path::Path;
use std::sync::Arc;
use test::Bencher;

fn config() -> Arc<Config> {
    Arc::new(Config::default())
}

fn matcher() -> Arc<Matcher> {
    Arc::new(Matcher::new())
}

fn play_out_aftermath(play_out_aftermath: bool) -> Arc<Config> {
    let mut config = Config::default();
    config.play_out_aftermath = play_out_aftermath;
    Arc::new(config)
}

#[test]
fn root_expands_the_children() {
    let game = Game::new(2, 0.5, KgsChinese);
    let root = Node::root(&game, Black, config());
    assert_eq!(4, root.children.len());
}

// expand()
#[test]
fn expand_doesnt_add_children_to_terminal_nodes() {
    let mut game = Game::new(5, 6.5, KgsChinese);
    game = game.play(Pass(Black)).unwrap();
    game = game.play(Pass(White)).unwrap();
    let mut node = Node::new(Pass(Black), config());
    node.expand(&game.board(), matcher());
    assert_eq!(0, node.children.len());
}

#[test]
fn expand_doesnt_add_children_if_threshold_not_met() {
    let game = Game::new(2, 0.5, KgsChinese);
    let mut node = Node::new(Pass(Black), config());
    node.plays = 0;
    node.expand(&game.board(), matcher());
    assert_eq!(0, node.children.len());
}

#[test]
fn expand_adds_children_if_threshold_is_met() {
    let game = Game::new(2, 0.5, KgsChinese);
    let mut node = Node::new(Pass(Black), config());
    node.plays = 2;
    node.expand(&game.board(), matcher());
    assert_eq!(4, node.children.len());
}

#[test]
fn expand_sets_the_descendant_count_if_the_node_was_expanded() {
    let game = Game::new(5, 6.5, KgsChinese);
    let board = game.board();
    let mut node = Node::new(Pass(Black), config());
    node.expand(&board,matcher());
    assert_eq!(25, node.descendants);
}

#[test]
fn expand_adds_pass_in_the_endgame() {
    let parser = Parser::from_path(Path::new("fixtures/sgf/endgame-black-wins.sgf")).unwrap();
    let game = parser.game().unwrap();
    let mut board = game.board();
    board.set_ruleset(KgsChinese);
    board.play(Pass(White)).unwrap();
    assert_eq!(Black, board.next_player());
    let mut node = Node::new(Pass(Black), config());
    node.expand(&board, matcher());
    let found_pass = node.children.iter().any(|node| node.m().is_pass());
    assert!(found_pass);
}

#[test]
fn expand_doesnt_add_pass_before_the_endgame() {
    let game = Game::new(5, 6.5, KgsChinese);
    let board = game.board();
    let mut node = Node::new(Pass(Black), config());
    node.expand(&board, matcher());
    let found_pass = node.children.iter().any(|node| node.m().is_pass());
    assert!(!found_pass);
}

#[test]
fn expand_doesnt_add_pass_if_we_are_losing_and_we_playout_the_aftermath() {
    let config = play_out_aftermath(true);
    let parser = Parser::from_path(Path::new("fixtures/sgf/endgame-black-wins.sgf")).unwrap();
    let game = parser.game().unwrap();
    let mut board = game.board();
    board.set_ruleset(KgsChinese);
    assert_eq!(White, board.next_player());
    let mut node = Node::new(Pass(White), config);
    node.expand(&board, matcher());
    let found_pass = node.children.iter().any(|node| node.m().is_pass());
    assert!(!found_pass);
}

#[test]
fn expand_adds_pass_if_we_are_losing_and_dont_playout_the_aftermath() {
    let config = play_out_aftermath(false);
    let parser = Parser::from_path(Path::new("fixtures/sgf/endgame-black-wins.sgf")).unwrap();
    let game = parser.game().unwrap();
    let mut board = game.board();
    board.set_ruleset(KgsChinese);
    assert_eq!(White, board.next_player());
    let mut node = Node::new(Pass(White), config);
    node.expand(&board, matcher());
    let found_pass = node.children.iter().any(|node| node.m().is_pass());
    assert!(found_pass);
}

#[test]
fn expand_adds_pass_if_we_are_winning_and_we_are_playing_out_the_aftermath() {
    let config = play_out_aftermath(true);
    let parser = Parser::from_path(Path::new("fixtures/sgf/endgame-black-wins.sgf")).unwrap();
    let game = parser.game().unwrap();
    let mut board = game.board();
    board.set_ruleset(KgsChinese);
    board.play(Pass(White)).unwrap();
    assert_eq!(Black, board.next_player());
    let mut node = Node::new(Pass(Black), config);
    node.expand(&board, matcher());
    let found_pass = node.children.iter().any(|node| node.m().is_pass());
    assert!(found_pass);
}

// find_leaf_and_expand()
#[test]
fn find_leaf_and_expand_expands_the_leaves() {
    let game = Game::new(2, 0.5, KgsChinese);
    let mut root = Node::root(&game, Black, config());
    for _ in 0..4 {
        root.find_leaf_and_expand(&game, matcher());
    }
    assert_eq!(4, root.children.len());
    assert!(root.children.iter().all(|n| n.children.len() == 3));
}

#[test]
fn find_leaf_and_expand_sets_play_on_the_root() {
    let game = Game::new(2, 0.5, KgsChinese);
    let mut root = Node::root(&game, Black, config());
    root.find_leaf_and_expand(&game, matcher());
    assert_eq!(2, root.plays);
}

#[test]
fn find_leaf_and_expand_returns_the_number_of_nodes_added() {
    let game = Game::new(2, 0.5, KgsChinese);
    let mut root = Node::root(&game, Black, config());
    let (_,_,_,count) = root.find_leaf_and_expand(&game, matcher());
    assert_eq!(3, count);
}

#[test]
fn the_root_needs_to_be_initialized_with_1_plays_for_correct_uct_calculations() {
    let game = Game::new(2, 0.5, KgsChinese);
    let root = Node::root(&game, Black, config());
    assert_eq!(1, root.plays);
    assert_eq!(1, root.wins);
 }

#[test]
fn no_super_ko_violations_in_the_children_of_the_root() {
    let parser = Parser::from_path(Path::new("fixtures/sgf/positional-superko.sgf")).unwrap();
    let game = parser.game().unwrap();
    let root = Node::root(&game, White, config());
    // Play(White, 2, 9) is a super ko violation
    assert!(root.children.iter().all(|n| n.m() != Play(White, 2, 9)));
}

#[test]
fn record_on_path_only_records_wins_for_the_correct_color() {
    let config = config();
    let grandchild = Node::new(Pass(Black), config.clone());
    let mut child = Node::new(Pass(White), config.clone());
    child.children = vec!(grandchild);
    let mut root = Node::new(Pass(Black), config.clone());
    root.children = vec!(child);

    root.record_on_path(&vec!(0, 0), Black, 0);
    assert_eq!(6, root.wins);
    assert_eq!(5, root.children[0].wins);
    assert_eq!(6, root.children[0].children[0].wins);

    root.record_on_path(&vec!(0, 0), White, 0);
    assert_eq!(6, root.wins);
    assert_eq!(6, root.children[0].wins);
    assert_eq!(6, root.children[0].children[0].wins);
}

#[test]
fn record_on_path_updates_the_descendant_counts() {
    let mut grandchild = Node::new(Pass(Black), config().clone());
    // The leaf already has the correct value set
    grandchild.descendants = 5;
    let mut child = Node::new(Pass(White), config().clone());
    child.children = vec!(grandchild);
    child.descendants = 1;
    let mut root = Node::new(Pass(Black), config().clone());
    root.children = vec!(child);
    root.descendants = 2;

    root.record_on_path(&vec!(0, 0), Black, 5);
    assert_eq!(7, root.descendants);
    assert_eq!(6, root.children[0].descendants);
    assert_eq!(5, root.children[0].children[0].descendants);
}

#[test]
fn find_child_returns_the_correct_child() {
    let mut root = Node::new(Pass(Black), config().clone());
    let child = Node::new(Play(White, 1, 1), config().clone());
    root.children = vec!(Node::new(Play(Black, 5, 5), config().clone()), child.clone(), Node::new(Play(Black, 3, 7), config().clone()));
    assert_eq!(child, root.find_child(Play(White, 1, 1)));
}

#[test]
fn new_sets_the_descendats_to_zero() {
    let node = Node::new(Pass(Black), config());
    assert_eq!(0, node.descendants);
}

// expand_root()
#[test]
fn expand_root_sets_the_correct_descendant_count_on_the_root() {
    let game = Game::new(5, 6.5, KgsChinese);
    let mut root = Node::new(Pass(Black), config());
    root.expand_root(&game);
    assert_eq!(25, root.descendants);
}

#[test]
fn expand_root_adds_pass_in_the_endgame() {
    let parser = Parser::from_path(Path::new("fixtures/sgf/endgame-black-wins.sgf")).unwrap();
    let mut game = parser.game().unwrap();
    game = game.play(Pass(White)).unwrap();
    let mut node = Node::new(Pass(Black), config());
    node.expand_root(&game);
    let found_pass = node.children.iter().any(|node| node.m().is_pass());
    assert!(found_pass);
}

#[test]
fn expand_root_doesnt_add_pass_before_the_endgame() {
    let game = Game::new(5, 6.5, KgsChinese);
    let mut node = Node::new(Pass(Black), config());
    node.expand_root(&game);
    let found_pass = node.children.iter().any(|node| node.m().is_pass());
    assert!(!found_pass);
}

#[test]
fn expand_root_doesnt_add_pass_if_we_are_losing_and_we_playout_the_aftermath() {
    let config = play_out_aftermath(true);
    let parser = Parser::from_path(Path::new("fixtures/sgf/endgame-black-wins.sgf")).unwrap();
    let game = parser.game().unwrap();
    let mut node = Node::new(Pass(White), config);
    node.expand_root(&game);
    let found_pass = node.children.iter().any(|node| node.m().is_pass());
    assert!(!found_pass);
}

#[test]
fn expand_root_adds_pass_if_we_are_losing_and_dont_playout_the_aftermath() {
    let config = play_out_aftermath(false);
    let parser = Parser::from_path(Path::new("fixtures/sgf/endgame-black-wins.sgf")).unwrap();
    let game = parser.game().unwrap();
    let mut node = Node::new(Pass(White), config);
    node.expand_root(&game);
    let found_pass = node.children.iter().any(|node| node.m().is_pass());
    assert!(found_pass);
}

#[test]
fn expand_root_adds_pass_if_we_are_winning_and_we_are_playing_out_the_aftermath() {
    let config = play_out_aftermath(true);
    let parser = Parser::from_path(Path::new("fixtures/sgf/endgame-black-wins.sgf")).unwrap();
    let mut game = parser.game().unwrap();
    game = game.play(Pass(White)).unwrap();
    let mut node = Node::new(Pass(Black), config);
    node.expand_root(&game);
    let found_pass = node.children.iter().any(|node| node.m().is_pass());
    assert!(found_pass);
}

// remove_illegal_children()
#[test]
fn remove_illegal_children_removes_superko_violations() {
    let parser = Parser::from_path(Path::new("fixtures/sgf/positional-superko.sgf")).unwrap();
    let game = parser.game().unwrap();
    let mut node = Node::new(Pass(White), config());
    // Play(White, 2, 9) is a super ko violation
    node.children.push(Node::new(Play(White, 2, 9), config()));
    node.remove_illegal_children(&game);
    assert!(node.children.iter().all(|n| n.m() != Play(White, 2, 9)));
}

#[test]
fn remove_illegal_children_removes_pass_if_the_other_color_wins_with_aftermath_turned_on() {
    let config = play_out_aftermath(true);
    let parser = Parser::from_path(Path::new("fixtures/sgf/endgame-black-wins.sgf")).unwrap();
    let game = parser.game().unwrap();
    let mut node = Node::new(Pass(Black), config.clone());
    node.children.push(Node::new(Pass(White), config.clone()));
    node.remove_illegal_children(&game);
    let found_pass = node.children.iter().any(|node| node.m().is_pass());
    assert!(!found_pass);
}

#[test]
fn remove_illegal_children_doesnt_remove_pass_if_we_are_winning() {
    let config = play_out_aftermath(true);
    let parser = Parser::from_path(Path::new("fixtures/sgf/endgame-black-wins.sgf")).unwrap();
    let mut game = parser.game().unwrap();
    game = game.play(Pass(White)).unwrap();
    let mut node = Node::new(Pass(White), config.clone());
    node.children.push(Node::new(Pass(Black), config.clone()));
    node.remove_illegal_children(&game);
    let found_pass = node.children.iter().any(|node| node.m().is_pass());
    assert!(found_pass);
}

#[test]
fn remove_illegal_children_doesnt_remove_pass_if_we_are_losing_but_dont_playout_aftermath() {
    let config = play_out_aftermath(false);
    let parser = Parser::from_path(Path::new("fixtures/sgf/endgame-black-wins.sgf")).unwrap();
    let game = parser.game().unwrap();
    let mut node = Node::new(Pass(Black), config.clone());
    node.children.push(Node::new(Pass(White), config.clone()));
    node.remove_illegal_children(&game);
    let found_pass = node.children.iter().any(|node| node.m().is_pass());
    assert!(found_pass);
}

#[bench]
fn full_uct_cycle_09x09(b: &mut Bencher) {
    full_uct_cycle(9, b);
}

#[bench]
fn full_uct_cycle_13x13(b: &mut Bencher) {
    full_uct_cycle(13, b);
}

#[bench]
fn full_uct_cycle_19x19(b: &mut Bencher) {
    full_uct_cycle(19, b);
}

fn full_uct_cycle(size: u8, b: &mut Bencher) {
    let game = Game::new(size, 6.5, KgsChinese);
    let mut cfg = Config::default();
    let matcher = matcher();
    cfg.play_out_aftermath = true;
    cfg.uct.priors.use_patterns = true;
    let config = Arc::new(cfg);
    let mut root = Node::root(&game, Black, config.clone());
    let playout = Playout::new(config.clone(), matcher.clone());
    let mut rng = weak_rng();
    b.iter(|| {
        let (path, moves, _, nodes_added) = root.find_leaf_and_expand(&game, matcher.clone());
        let mut b = game.board();
        for &m in moves.iter() {
            b.play_legal_move(m);
        }
        let playout_result = playout.run(&mut b, None, &mut rng);
        let winner = playout_result.winner();
        root.record_on_path(&path, winner, nodes_added);
    });
}
