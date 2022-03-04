use crate::*;

pub struct GameOverPlugin;

impl Plugin for GameOverPlugin {
    fn build(&self, app: &mut App) {
        app.add_system_set(SystemSet::on_enter(GameState::GameOver).with_system(game_over_setup))
            .add_system_set(
                SystemSet::on_exit(GameState::GameOver)
                    .with_system(despawn_components_system::<GameOverComponent>),
            );
    }
}

#[derive(Component)]
struct GameOverComponent;

/// Sets up the game over screen.
fn game_over_setup() {
    todo!(); //TODO
}
