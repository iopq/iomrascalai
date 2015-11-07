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

use board::Color;
use board::Move;
use config::Config;
use engine::Engine;
use game::Game;
use timer::Timer;

use std::sync::Arc;
use std::sync::mpsc::channel;
use std::sync::mpsc::Sender;
use std::thread;
use std::thread::sleep_ms;
use thread_scoped::scoped;

mod test;

pub struct EngineController<'a> {
    config: Arc<Config>,
    engine: Box<Engine + 'a>,
}

impl<'a> EngineController<'a> {

    pub fn new<'b>(config: Arc<Config>, engine: Box<Engine + 'b>) -> EngineController<'b> {
        EngineController {
            config: config,
            engine: engine,
        }
    }

    pub fn reset(&mut self) {
        self.engine.reset();
    }

    pub fn run_and_return_move(&mut self, color: Color, game: &Game, timer: &Timer, send_move: Sender<Move>) -> usize {
        let budget = self.budget(timer, game);
        let (send_move_to_controller, receive_move_from_engine) = channel();
        let (send_signal_to_engine, receive_signal_from_controller) = channel::<()>();
        // Saving the guard into a variable is necessary. Otherwise
        // the code blocks right here.
        unsafe {
            let ste_config = self.config.clone();
            let _guard = scoped(|| {
                self.engine.gen_move(color, budget, game, send_move_to_controller, receive_signal_from_controller);
            });

            let (send_time_up_to_controller, receive_time_up) = channel();
            thread::spawn(move || {
                sleep_ms(budget);
                match send_time_up_to_controller.send(()) {
                    Ok(_) => {}
                    Err(_) => {
                        // This is expected to fail whenever the
                        // engine returns before the allotted time is
                        // up.
                    }
                }
            });
            select!(
                r = receive_move_from_engine.recv() => {
                    let (m, playouts) = r.unwrap();
                    send_move.send(m).unwrap();
                    playouts
                },
                _ = receive_time_up.recv() => {
                    match send_signal_to_engine.send(()) {
                        Ok(_) => {},
                        Err(e) => {
                            ste_config.log(format!("[DEBUG] sending time up to engine failed with {:?}", e));
                        }
                    }
                    let (m, playouts) = receive_move_from_engine.recv().unwrap();
                    send_move.send(m).unwrap();
                    playouts
                }
            )
        }
    }

    fn budget(&self, timer: &Timer, game: &Game) -> u32 {
        let budget = timer.budget(game);
        self.config.log(
            format!("Thinking for {}ms ({}ms time left)", budget, timer.main_time_left()));
        budget
    }

}
