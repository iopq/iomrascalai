/************************************************************************
 *                                                                      *
 * Copyright 2014 Thomas Poinsot, Urban Hafner                          *
 * Copyright 2015 Urban Hafner, Thomas Poinsot, Igor Polyakov           *
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
#![allow(non_camel_case_types)]

use board::Color;
use board::IllegalMove;
use board::Move;
use engine::Engine;
use engine::EngineController;
use game::Game;
use ruleset::Ruleset;
use sgf::parser::Parser;
use timer::Timer;
use strenum::Strenum;

pub mod driver;
mod test;

strenum! {
    knownCommands => 
        boardsize,
        clear_board,
        final_score,
        genmove,
        known_command,
        komi,
        list_commands,
        loadsgf,
        name,
        play,
        protocol_version,
        quit,
        showboard,
        time_left,
        time_settings,
        version
}

pub enum Command {
    BoardSize,
    ClearBoard,
    Empty,
    Error,
    ErrorMessage(String),
    FinalScore(String),
    GenMove(String),
    GenMoveError(Move, IllegalMove),
    KnownCommand(bool),
    Komi,
    ListCommands(String),
    LoadSgf,
    Name,
    Play,
    PlayError(Move, IllegalMove),
    ProtocolVersion,
    Quit,
    ShowBoard(String),
    TimeLeft,
    TimeSettings,
    Version,
}

pub struct GTPInterpreter<'a> {
    controller: EngineController<'a>,
    game: Game,
    ruleset: Ruleset,
    timer: Timer,
}

impl<'a> GTPInterpreter<'a> {
    pub fn new<'b>(ruleset: Ruleset, engine: Box<Engine + 'b>) -> GTPInterpreter<'b> {
        let komi      = 6.5;
        let boardsize = 19;
        GTPInterpreter {
            controller: EngineController::new(engine),
            game: Game::new(boardsize, komi, ruleset),
            ruleset: ruleset,
            timer: Timer::new(),
        }
    }

    pub fn game<'b>(&'b self) -> &'b Game {
        &self.game
    }

    pub fn komi(&self) -> f32 {
        self.game.komi()
    }

    pub fn ruleset(&self) -> Ruleset {
        self.ruleset
    }

    pub fn main_time(&self) -> i64 {
        self.timer.main_time()
    }

    pub fn byo_time(&self) -> i64 {
        self.timer.byo_time()
    }

    pub fn byo_stones(&self) -> i32 {
        self.timer.byo_stones()
    }

    pub fn boardsize(&self) -> u8 {
        self.game.size()
    }

    pub fn read(&mut self, input: &str) -> Command {
        let preprocessed = self.preprocess(input);

        if preprocessed.len() == 0 {return Command::Empty};

        let command: Vec<&str> = preprocessed.as_slice().split(' ').collect();

        match <knownCommands>::enumify(command[0]) {
            Some(knownCommands::name)             => Command::Name,
            Some(knownCommands::version)          => Command::Version,
            Some(knownCommands::protocol_version) => Command::ProtocolVersion,
            Some(knownCommands::list_commands)    => Command::ListCommands(<knownCommands>::stringify()),
            Some(knownCommands::known_command)    =>
                match command.get(1) {
                	Some(comm) => Command::KnownCommand(<knownCommands>::enumify(&comm).is_some()),
                	None => Command::KnownCommand(false)
            	},
            Some(knownCommands::boardsize)        =>
                match command.get(1) {
                	Some(comm) => match comm.parse::<u8>() {
                        Ok(size) => {
                            self.game = Game::new(size, self.komi(), self.ruleset());
                            Command::BoardSize
                        },
                        Err(_) => Command::Error
                    },
                	None => Command::Error
            	},
            Some(knownCommands::clear_board)      => {
                self.game = Game::new(self.boardsize(), self.komi(), self.ruleset());
                self.timer.reset();
                Command::ClearBoard
            },
            Some(knownCommands::komi)             =>
                match command.get(1) {
                    Some(comm) =>
                        match comm.parse::<f32>() {
                            Ok(komi) => {
                                self.game.set_komi(komi);
                                Command::Komi
                            },
                            Err(_) => Command::Error
                        },
                    None => Command::Error
                },
            Some(knownCommands::genmove)          =>
            	match command.get(1) {
            		Some(comm) => {
            			let color = Color::from_gtp(comm);
                        let m = self.controller.run_and_return_move(color, &self.game, &mut self.timer);
                        match self.game.clone().play(m) {
                            Ok(g) => {
                                self.game = g;
                                self.timer.stop();
                                Command::GenMove(m.to_gtp())
                            },
                            Err(e) => {
                                Command::GenMoveError(m, e)
                            }
                        }
                    },
                    None => Command::Error
        		},
            Some(knownCommands::play)             => 
            match command.get(2) {
            	Some(second) => {
                    let m = Move::from_gtp(command[1], second); //command[1] should be there
                    match self.game.clone().play(m) {
                        Ok(g) => {
                            self.game = g;
                            Command::Play
                        },
                        Err(e) => {
                            Command::PlayError(m, e)
                        }
                    }
                },
            	None => Command::Error
        	},
            Some(knownCommands::showboard)   => Command::ShowBoard(format!("\n{}", self.game)),
            Some(knownCommands::quit)        => Command::Quit,
            Some(knownCommands::final_score) => Command::FinalScore(format!("{}", self.game.score())),
            Some(knownCommands::time_settings) => match command.get(3) {
            	Some(third) => {
            		//command[1] and command[2] should be there
                    match (command[1].parse::<i64>(), command[2].parse::<i64>(), third.parse::<i32>()) {
                        (Ok(main), Ok(byo), Ok(stones)) => {
                            self.timer.setup(main, byo, stones);
                            Command::TimeSettings
                        }
                        _ => Command::Error
                    }
                },
            	None => Command::Error
        	},
            Some(knownCommands::time_left) => match command.get(3) {
            	Some(third) => {
            		//command[2] should be there
            		//TODO: seems wrong, missing a color
                    match (command[2].parse::<i64>(), third.parse::<i32>()) {
                        (Ok(time), Ok(stones)) => {
                            self.timer.update(time, stones);
                            Command::TimeLeft
                        },
                        _ => Command::Error
                    }
                },
            	None => Command::Error
        	},
            Some(knownCommands::loadsgf) => match command.get(1) {
            	Some(comm) => {
                    let filename = comm;
                    
                    match Parser::attempt_from_path(Path::new(filename)) {
                    	Ok(parser) => {
                    		let game = parser.game();
                            match game {
                                Ok(g) => {
                                    self.game = g;
                                    Command::LoadSgf
                                },
                                Err(_) => Command::ErrorMessage(String::from_str("cannot load file"))
                            }
                		},
                    	Err(_) => Command::ErrorMessage(String::from_str("cannot load file"))
                	}
                },
            	None => Command::Error
        	},
            None             => Command::Error
        }
    }

    fn preprocess(&self, input: &str) -> String {
        // Convert tab to space
        let horizontal_tab = regex!(r"\t");
        let without_tabs = horizontal_tab.replace_all(input, " ");
        // Remove all control characters
        let cntrls = regex!(r"[:cntrl:]");
        let without_ctrls = cntrls.replace_all(without_tabs.as_slice(), "");
        // Then we remove anything after a #
        let comment = regex!(r"#.*");
        let without_comment = comment.replace(without_ctrls.as_slice(), "");
        // We remove the whitespaces before/after the string
        without_comment.as_slice().trim().to_string()
    }
}
