use bevy::prelude::*;

/// Ecosystem tuning parameters for Step 8 - Easy balance adjustment
#[derive(Resource)]
pub struct EcosystemTuning {
    // Resource regeneration rates
    pub plant_regeneration_rate: f32,
    pub water_regeneration_rate: f32,
    pub sunlight_regeneration_rate: f32,
    pub mineral_regeneration_rate: f32,
    pub detritus_regeneration_rate: f32,
    pub prey_regeneration_rate: f32,

    // Resource decay rates
    pub plant_decay_rate: f32,
    pub water_decay_rate: f32,
    pub sunlight_decay_rate: f32,
    pub mineral_decay_rate: f32,
    pub detritus_decay_rate: f32,
    pub prey_decay_rate: f32,

    // Consumption rates
    pub consumption_rate_base: f32,
    pub energy_conversion_efficiency: f32,
    pub decomposer_efficiency_multiplier: f32,

    // Metabolism tuning
    pub base_metabolism_multiplier: f32,
    pub movement_cost_multiplier: f32,

    // Reproduction tuning
    pub reproduction_chance_multiplier: f32,
    pub min_reproduction_cooldown: f32,
    pub max_reproduction_cooldown: f32,

    // Spawn parameters
    pub initial_spawn_count: usize,
    
    // Speciation
    pub speciation_threshold: f32,
}

impl Default for EcosystemTuning {
    fn default() -> Self {
        Self {
            // Balanced regeneration rates (tuned for Step 8)
            plant_regeneration_rate: 0.08,
            water_regeneration_rate: 0.12,
            sunlight_regeneration_rate: 0.15,
            mineral_regeneration_rate: 0.05,
            detritus_regeneration_rate: 0.03,
            prey_regeneration_rate: 0.02,

            // Decay rates (resources naturally decay over time)
            plant_decay_rate: 0.01,
            water_decay_rate: 0.005,
            sunlight_decay_rate: 0.02, // Sunlight cycles quickly
            mineral_decay_rate: 0.001,   // Minerals decay very slowly
            detritus_decay_rate: 0.015,  // Detritus breaks down
            prey_decay_rate: 0.02,       // Prey resources decay quickly

            // Consumption
            consumption_rate_base: 5.0,
            energy_conversion_efficiency: 0.3,
            decomposer_efficiency_multiplier: 0.5,

            // Metabolism
            base_metabolism_multiplier: 1.0,
            movement_cost_multiplier: 1.0,

            // Reproduction
            reproduction_chance_multiplier: 0.1, // 10% chance per frame when conditions met
            min_reproduction_cooldown: 350.0,
            max_reproduction_cooldown: 2400.0,

            // Spawn
            initial_spawn_count: 100,

            // Speciation
            speciation_threshold: 0.15,
        }
    }
}

impl EcosystemTuning {
    /// Create balanced preset for stable ecosystem
    pub fn balanced() -> Self {
        Self::default()
    }

    /// Create preset for fast evolution (higher mutation, faster reproduction)
    pub fn fast_evolution() -> Self {
        let mut tuning = Self::default();
        tuning.reproduction_chance_multiplier = 0.15; // 15% chance
        tuning.min_reproduction_cooldown = 200.0;
        tuning.max_reproduction_cooldown = 1200.0;
        tuning.plant_regeneration_rate = 0.12; // More resources
        tuning
    }

    /// Create preset for slow, stable ecosystem (lower reproduction, higher resources)
    pub fn stable() -> Self {
        let mut tuning = Self::default();
        tuning.reproduction_chance_multiplier = 0.05; // 5% chance
        tuning.min_reproduction_cooldown = 500.0;
        tuning.max_reproduction_cooldown = 3000.0;
        tuning.plant_regeneration_rate = 0.15; // More resources for stability
        tuning.water_regeneration_rate = 0.18;
        tuning
    }

    /// Create preset for competitive ecosystem (scarce resources, faster decay)
    pub fn competitive() -> Self {
        let mut tuning = Self::default();
        tuning.plant_regeneration_rate = 0.05; // Scarce resources
        tuning.water_regeneration_rate = 0.08;
        tuning.plant_decay_rate = 0.02; // Faster decay
        tuning.consumption_rate_base = 7.0; // Faster consumption
        tuning
    }
}

