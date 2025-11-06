mod components;
mod systems;
mod genetics;

use bevy::prelude::*;
pub use components::*;
pub use genetics::*;

pub struct OrganismPlugin;

impl Plugin for OrganismPlugin {
    fn build(&self, app: &mut App) {
        app
            .init_resource::<systems::TrackedOrganism>()
            .add_systems(Startup, systems::spawn_initial_organisms)
            .add_systems(Update, (
                systems::update_metabolism,
                systems::update_movement,
                systems::update_age,
                systems::handle_reproduction,
                systems::handle_death,
                systems::log_tracked_organism,
            ));
    }
}

