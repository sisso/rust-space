use domain_b::game::utils::DeltaTime;
use domain_b::game::Game;

fn main() {
    let mut game = Game::new();
    game.tick(DeltaTime(1.0));
}
