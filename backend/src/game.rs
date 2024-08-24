use std::fmt::Debug;
use serde::Serialize;

#[derive(Clone, Copy, Debug, PartialEq, Serialize)]
pub enum Player {X, O}

#[derive(Clone, Debug)]
pub struct Game {
    current_player: Option<Player>,
    pub board: [Option<Player>; 9]
}

impl Game {
    pub fn new() -> Self {
        let board: [Option<Player>; 9] = [None; 9];
        let current_player: Option<Player> = Some(Player::X);
        Game { board, current_player }
    }
    pub fn get_board(&self) -> &[Option<Player>; 9] {
        &self.board
    }
    pub fn print_board(&self) {
        for x in 0..=2 {
            for y in 0..=2 {
                let label: &str = if let Some(player) = &self.board[x*3 + y] {
                    match player {
                        Player::X => "X",
                        Player::O => "O",
                    }
                } else { " " };
                let divider: &str = if y != 2 { "|" } else { " " };
                print!(" {} {}", label, divider);
            }
            println!("{}", if x != 2 { "\n-----------" } else { "\n" });
        }
    }
    pub fn move_player(&mut self, player: Player, position: usize) -> Result<(), &str> {
        if position > 10 { return Err("Position is out of bounds"); }
        if self.board[position].is_some() {
            return Err("A player is already at this location");
        }
        if player != *self.current_player.as_ref().unwrap() {
            return Err("Cannot move the player. It is not their turn.");
        }
        self.board[position].clone_from(&self.current_player);
        self.current_player = match player {
            Player::X => Some(Player::O),
            Player::O => Some(Player::X),
        };
        self.print_board();
        Ok(())
    }
}

impl Default for Game {
    fn default() -> Self {
        Self::new()
    }
}


#[cfg(test)]
mod tests {
    use crate::{Game, Player};
    #[test]
    fn test_board() {
        let mut game: Game = Game::new();
        assert_eq!(game.current_player, Some(Player::X));
        assert!(game.board.iter().all(|x| x.is_none()));
        let _ = game.move_player(Player::X, 0);
        assert_eq!(game.board[0], Some(Player::X));
        let _ = game.move_player(Player::O, 4);
        assert_eq!(game.board[4], Some(Player::O));
        let _ = game.move_player(Player::X, 8);
        assert_eq!(game.board[8], Some(Player::X));
        let _ = game.move_player(Player::O, 2);
        assert_eq!(game.board[2], Some(Player::O));
        let _ = game.move_player(Player::X, 6);
        assert_eq!(game.board[6], Some(Player::X));
        let _ = game.move_player(Player::O, 7);
        assert_eq!(game.board[7], Some(Player::O));
        let _ = game.move_player(Player::X, 3);
        assert_eq!(game.board[3], Some(Player::X));
    }
}