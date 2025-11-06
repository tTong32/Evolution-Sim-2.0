use bevy::prelude::*;
use crate::organisms::components::*;
use crate::organisms::genetics::{Genome, traits, DEFAULT_MUTATION_RATE};
use crate::organisms::behavior::*;
use crate::world::{WorldGrid, ResourceType};
use rand::Rng;

use std::fs::{File, OpenOptions};
use std::io::{Write, BufWriter};
use std::path::PathBuf;

/// Resource to track which organism we're logging
#[derive(Resource)]
pub struct TrackedOrganism {
    entity: Option<Entity>,
    log_counter: u32,
    csv_writer: Option<BufWriter<File>>,
    csv_path: PathBuf,
    header_written: bool,
}

impl Default for TrackedOrganism {
    fn default() -> Self {
        // Create data/logs directory if it doesn't exist
        let logs_dir = PathBuf::from("data/logs");
        if !logs_dir.exists() {
            std::fs::create_dir_all(&logs_dir).expect("Failed to create logs directory");
        }
        
        // Create CSV file with timestamp
        let timestamp = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs();
        let csv_path = logs_dir.join(format!("organism_tracking_{}.csv", timestamp));
        
        Self {
            entity: None,
            log_counter: 0,
            csv_writer: None,
            csv_path,
            header_written: false,
        }
    }
}

/// Spawn initial organisms in the world
pub fn spawn_initial_organisms(
    mut commands: Commands,
    mut tracked: ResMut<TrackedOrganism>,
    _world_grid: Res<WorldGrid>,
) {
    info!("Spawning initial organisms...");
    
    let mut rng = rand::thread_rng();
    let spawn_count = 100; // Start with 100 organisms
    
    // Spawn organisms randomly within initialized chunks
    // Chunks are from -1 to 1, each chunk is 64x64 cells
    let world_size = 3 * 64; // 3 chunks * 64 cells
    let spawn_range = world_size as f32 / 2.0; // -range to +range
    
    let mut first_entity = None;
    
    for i in 0..spawn_count {
        let x = rng.gen_range(-spawn_range..spawn_range);
        let y = rng.gen_range(-spawn_range..spawn_range);
        
        // Create random genome for this organism
        let genome = Genome::random();
        
        // Express traits from genome
        let size = traits::express_size(&genome);
        let max_energy = traits::express_max_energy(&genome);
        let metabolism_rate = traits::express_metabolism_rate(&genome);
        let movement_cost = traits::express_movement_cost(&genome);
        let reproduction_cooldown = traits::express_reproduction_cooldown(&genome) as u32;
        
        let organism_type = match rng.gen_range(0..3) {
            0 => OrganismType::Producer,
            1 => OrganismType::Consumer,
            _ => OrganismType::Decomposer,
        };
        
        // Random initial velocity
        let vel_x = rng.gen_range(-10.0..10.0);
        let vel_y = rng.gen_range(-10.0..10.0);
        
        let entity = commands.spawn((
            Position::new(x, y),
            Velocity::new(vel_x, vel_y),
            Energy::new(max_energy),
            Age::new(),
            Size::new(size),
            Metabolism::new(metabolism_rate, movement_cost),
            ReproductionCooldown::new(reproduction_cooldown),
            genome,
            SpeciesId::new(0), // All start as same species for now
            organism_type,
            Behavior::new(),
            Alive,
        )).id();
        
        // Track the first organism spawned
        if i == 0 {
            first_entity = Some(entity);
        }
    }
    
    // Set the first organism as the tracked one
    if let Some(entity) = first_entity {
        tracked.entity = Some(entity);
        
        // Initialize CSV writer
        let file = OpenOptions::new()
            .create(true)
            .append(true)
            .open(&tracked.csv_path)
            .expect("Failed to open CSV file for writing");
        tracked.csv_writer = Some(BufWriter::new(file));
        
        info!("[TRACKED] Started tracking organism entity: {:?}", entity);
        info!("[TRACKED] CSV logging to: {}", tracked.csv_path.display());
        info!("[TRACKED] Logging will begin after 10 ticks...");
    }
    
    info!("Spawned {} organisms", spawn_count);
}

/// Update metabolism - organisms consume energy over time
/// Uses traits from genome (if available) or falls back to Metabolism component
pub fn update_metabolism(
    mut query: Query<(&mut Energy, &Velocity, &Metabolism, &Size, Option<&Genome>)>,
    time: Res<Time>,
) {
    let dt = time.delta_seconds();
    
    for (mut energy, velocity, metabolism, size, genome_opt) in query.iter_mut() {
        // Use genome traits if available, otherwise use Metabolism component
        let (base_rate, movement_cost_mult) = if let Some(genome) = genome_opt {
            (traits::express_metabolism_rate(genome), traits::express_movement_cost(genome))
        } else {
            (metabolism.base_rate, metabolism.movement_cost)
        };
        
        // Base metabolic cost (proportional to size)
        let base_cost = base_rate * size.value() * dt;
        
        // Movement cost (proportional to speed)
        let speed = velocity.0.length();
        let movement_cost = speed * movement_cost_mult * dt;
        
        // Total energy consumed
        let total_cost = base_cost + movement_cost;
        
        // Deduct energy
        energy.current -= total_cost;
        energy.current = energy.current.max(0.0);
    }
}

/// Update behavior decisions based on sensory input and organism state
pub fn update_behavior(
    mut query: Query<(
        Entity,
        &Position,
        &mut Behavior,
        &Energy,
        &Genome,
        &SpeciesId,
        &OrganismType,
        &Size,
    ), With<Alive>>,
    world_grid: Res<WorldGrid>,
    organism_query: Query<(Entity, &Position, &SpeciesId, &OrganismType, &Size, &Energy), With<Alive>>,
    time: Res<Time>,
) {
    let dt = time.delta_seconds();
    
    for (entity, position, mut behavior, energy, genome, species_id, organism_type, size) in query.iter_mut() {
        // Update state time
        behavior.state_time += dt;
        
        // Get sensory range from genome
        let sensory_range = traits::express_sensory_range(genome);
        
        // Collect sensory data
        let sensory = collect_sensory_data(
            entity,
            position.0,
            sensory_range,
            *species_id,
            *organism_type,
            size.value(),
            &world_grid,
            &organism_query,
        );
        
        // Make behavior decision
        let (new_state, target_entity, target_position) = decide_behavior(
            energy,
            genome,
            *organism_type,
            &sensory,
            behavior.state,
            behavior.state_time,
        );
        
        // Update behavior state
        behavior.set_state(new_state);
        behavior.target_entity = target_entity;
        behavior.target_position = target_position;
    }
}

/// Update organism movement based on behavior state
pub fn update_movement(
    mut query: Query<(
        &mut Position, 
        &mut Velocity, 
        &Behavior,
        &Energy, 
        &Size, 
        &Genome,
        &OrganismType,
        Entity
    ), With<Alive>>,
    time: Res<Time>,
    tracked: ResMut<TrackedOrganism>,
) {
    let dt = time.delta_seconds();
    let time_elapsed = time.elapsed_seconds();
    
    for (mut position, mut velocity, behavior, energy, _size, genome, organism_type, entity) in query.iter_mut() {
        // Skip if dead
        if energy.is_dead() {
            velocity.0 = Vec2::ZERO;
            continue;
        }
        
        // Calculate velocity based on behavior state
        let desired_velocity = calculate_behavior_velocity(
            behavior,
            position.0,
            genome,
            *organism_type,
            energy,
            time_elapsed,
        );
        
        // Smooth velocity transitions (lerp for smoother movement)
        let lerp_factor = 0.3; // How quickly velocity changes
        velocity.0 = velocity.0.lerp(desired_velocity, lerp_factor);
        
        // Apply velocity damping (friction) for wandering/resting
        if behavior.state == BehaviorState::Wandering || behavior.state == BehaviorState::Resting {
            velocity.0 *= 0.98;
        }
        
        // Update position
        position.0 += velocity.0 * dt;
        
        // Simple boundary checking (keep organisms within reasonable bounds)
        let max_pos = 200.0;
        position.0.x = position.0.x.clamp(-max_pos, max_pos);
        position.0.y = position.0.y.clamp(-max_pos, max_pos);
        
        if tracked.entity == Some(entity) && behavior.state_time < dt * 2.0 {
            // Log behavior changes
            info!("[TRACKED] Behavior: {:?}, Velocity: ({:.2}, {:.2}), Speed: {:.2}", 
                  behavior.state, velocity.0.x, velocity.0.y, velocity.0.length());
        }
    }
}

/// Handle eating behavior - consume resources or prey
pub fn handle_eating(
    mut query: Query<(
        Entity,
        &Position,
        &mut Energy,
        &Behavior,
        &OrganismType,
        &Size,
    ), With<Alive>>,
    mut world_grid: ResMut<WorldGrid>,
    _organism_query: Query<(&Position, &mut Energy, &Size), (With<Alive>, Without<Behavior>)>,
    time: Res<Time>,
) {
    let dt = time.delta_seconds();
    let consumption_rate = 5.0; // Resources consumed per second
    let energy_conversion_efficiency = 0.3; // 30% of consumed resources -> energy
    
    for (_entity, position, mut energy, behavior, organism_type, _size) in query.iter_mut() {
        if behavior.state != BehaviorState::Eating {
            continue;
        }
        
        // Get current cell
        if let Some(cell) = world_grid.get_cell_mut(position.x(), position.y()) {
            let consumed = match organism_type {
                OrganismType::Producer => {
                    // Producers consume sunlight, water, minerals
                    let sunlight = cell.get_resource(ResourceType::Sunlight).min(consumption_rate * dt);
                    let water = cell.get_resource(ResourceType::Water).min(consumption_rate * dt * 0.5);
                    let mineral = cell.get_resource(ResourceType::Mineral).min(consumption_rate * dt * 0.2);
                    
                    cell.set_resource(ResourceType::Sunlight, cell.get_resource(ResourceType::Sunlight) - sunlight);
                    cell.set_resource(ResourceType::Water, cell.get_resource(ResourceType::Water) - water);
                    cell.set_resource(ResourceType::Mineral, cell.get_resource(ResourceType::Mineral) - mineral);
                    
                    (sunlight + water + mineral) * energy_conversion_efficiency
                }
                OrganismType::Consumer => {
                    // Consumers consume plants or prey resources
                    let plant = cell.get_resource(ResourceType::Plant).min(consumption_rate * dt);
                    let prey_resource = cell.get_resource(ResourceType::Prey).min(consumption_rate * dt);
                    
                    cell.set_resource(ResourceType::Plant, cell.get_resource(ResourceType::Plant) - plant);
                    cell.set_resource(ResourceType::Prey, cell.get_resource(ResourceType::Prey) - prey_resource);
                    
                    (plant + prey_resource * 2.0) * energy_conversion_efficiency // Prey is more nutritious
                }
                OrganismType::Decomposer => {
                    // Decomposers consume detritus
                    let detritus = cell.get_resource(ResourceType::Detritus).min(consumption_rate * dt);
                    
                    cell.set_resource(ResourceType::Detritus, cell.get_resource(ResourceType::Detritus) - detritus);
                    
                    detritus * energy_conversion_efficiency * 0.5 // Less efficient
                }
            };
            
            // Add energy (clamped to max)
            energy.current = (energy.current + consumed).min(energy.max);
        }
        
    }
}

/// Update organism age and reproduction cooldown
pub fn update_age(
    mut query: Query<(&mut Age, &mut ReproductionCooldown)>,
) {
    for (mut age, mut cooldown) in query.iter_mut() {
        age.increment();
        cooldown.decrement();
    }
}

/// Handle reproduction - both asexual and sexual
pub fn handle_reproduction(
    mut commands: Commands,
    mut query: Query<(
        Entity,
        &Position,
        &mut Energy,
        &mut ReproductionCooldown,
        &Genome,
        &SpeciesId,
        &OrganismType,
    ), With<Alive>>,
) {
    let mut rng = rand::thread_rng();
    
    // First pass: collect all organism data for mate finding
    let all_organisms: Vec<_> = query.iter().map(|(e, p, _, _, g, s, o)| {
        (e, *p, g.clone(), *s, *o)
    }).collect();
    
    // Second pass: find candidates and their mates
    let mut reproduction_events = Vec::new();
    
    for (entity, position, energy, cooldown, genome, species_id, org_type) in query.iter() {
        if !cooldown.is_ready() {
            continue;
        }
        
        let reproduction_threshold = traits::express_reproduction_threshold(genome);
        if energy.ratio() < reproduction_threshold {
            continue;
        }
        
        // Only attempt reproduction with a probability (not guaranteed every frame)
        // This prevents mass reproduction when cooldown expires
        if !rng.gen_bool(0.1) { // 10% chance per frame when conditions are met
            continue;
        }
        
        // Decide between asexual and sexual reproduction
        let use_sexual = rng.gen_bool(0.3);
        
        let offspring_genome = if use_sexual {
            // Sexual reproduction - find a mate
            let sensory_range = traits::express_sensory_range(genome);
            let mut mate_opt = None;
            
            for (other_entity, other_pos, other_genome, other_species, _) in &all_organisms {
                if *other_entity == entity {
                    continue; // Skip self
                }
                
                if *other_species != *species_id {
                    continue; // Only mate with same species
                }
                
                let distance = (position.0 - other_pos.0).length();
                if distance <= sensory_range {
                    mate_opt = Some(other_genome.clone());
                    break;
                }
            }
            
            if let Some(mate_genome) = mate_opt {
                Genome::crossover(genome, &mate_genome, DEFAULT_MUTATION_RATE)
            } else {
                genome.clone_with_mutation(DEFAULT_MUTATION_RATE)
            }
        } else {
            genome.clone_with_mutation(DEFAULT_MUTATION_RATE)
        };
        
        reproduction_events.push((entity, *position, offspring_genome, *species_id, *org_type));
    }
    
    // Third pass: actually reproduce (mutable access)
    for (entity, position, offspring_genome, species_id, org_type) in reproduction_events {
        if let Ok((_, _, mut energy, mut cooldown, _genome, _, _)) = query.get_mut(entity) {
            // Express traits from offspring genome
            let size = traits::express_size(&offspring_genome);
            let max_energy = traits::express_max_energy(&offspring_genome);
            let metabolism_rate = traits::express_metabolism_rate(&offspring_genome);
            let movement_cost = traits::express_movement_cost(&offspring_genome);
            let reproduction_cooldown = traits::express_reproduction_cooldown(&offspring_genome) as u32;
            
            // Spawn offset from parent (small random offset)
            let offset_x = rng.gen_range(-5.0..5.0);
            let offset_y = rng.gen_range(-5.0..5.0);
            
            // Spawn offspring
            commands.spawn((
                Position::new(position.0.x + offset_x, position.0.y + offset_y),
                Velocity::new(0.0, 0.0),
                Energy::new(max_energy * 0.5), // Start with half energy
                Age::new(),
                Size::new(size),
                Metabolism::new(metabolism_rate, movement_cost),
                ReproductionCooldown::new(reproduction_cooldown),
                offspring_genome,
                species_id, // Inherit species ID
                org_type,
                Behavior::new(),
                Alive,
            ));
            
            // Deduct energy from parent (reproduction cost)
            energy.current *= 0.7; // Lose 30% of energy
            
            // Reset cooldown
            cooldown.reset(reproduction_cooldown);
            
            info!("Organism reproduced! New offspring spawned near parent");
        }
    }
}

/// Handle organism death (remove entities with zero energy)
pub fn handle_death(
    mut commands: Commands,
    mut tracked: ResMut<TrackedOrganism>,
    query: Query<(Entity, &Energy), With<Alive>>,
) {
    for (entity, energy) in query.iter() {
        if energy.is_dead() {
            if tracked.entity == Some(entity) {
                info!("[TRACKED] Organism died! Final energy: {:.2}", energy.current);
                tracked.entity = None; // Clear tracking
            }
            info!("Organism died at energy level: {:.2}", energy.current);
            commands.entity(entity).despawn();
        }
    }
}

/// Log tracked organism information periodically
pub fn log_tracked_organism(
    tracked: ResMut<TrackedOrganism>,
    query: Query<(Entity, &Position, &Velocity, &Energy, &Age, &Size, &OrganismType, &Behavior, &Genome), With<Alive>>,
) {
    let mut tracked_mut = tracked;
    tracked_mut.log_counter += 1;
    
    // Log every 10 ticks for more frequent output (change to 60 for less frequent)
    if tracked_mut.log_counter % 10 != 0 {
        return;
    }
    
    if let Some(entity) = tracked_mut.entity {
        if let Ok((_entity, position, velocity, energy, age, size, org_type, behavior, genome)) = query.get(entity) {
            let speed = velocity.0.length();
            let behavior_state = format!("{:?}", behavior.state);
            let sensory_range = traits::express_sensory_range(genome);
            let aggression = traits::express_aggression(genome);
            let boldness = traits::express_boldness(genome);
            
            // Format target information
            let target_info = if let Some(target_pos) = behavior.target_position {
                format!("({:.1}, {:.1})", target_pos.x, target_pos.y)
            } else {
                "None".to_string()
            };
            
            // Console logging
            info!(
                "[TRACKED ORGANISM] Tick: {} | Pos: ({:.2}, {:.2}) | Vel: ({:.2}, {:.2}) | Speed: {:.2} | Energy: {:.2}/{:.2} ({:.1}%) | Age: {} | Size: {:.2} | Type: {:?} | Behavior: {} | StateTime: {:.1}s | Target: {} | SensoryRange: {:.1} | Aggression: {:.2} | Boldness: {:.2}",
                tracked_mut.log_counter,
                position.0.x,
                position.0.y,
                velocity.0.x,
                velocity.0.y,
                speed,
                energy.current,
                energy.max,
                energy.ratio() * 100.0,
                age.0,
                size.value(),
                org_type,
                behavior_state,
                behavior.state_time,
                target_info,
                sensory_range,
                aggression,
                boldness
            );
            
            // CSV logging
            let needs_header = !tracked_mut.header_written;
            let tick = tracked_mut.log_counter;
            
            if let Some(ref mut writer) = tracked_mut.csv_writer {
                // Write CSV header if not written yet
                if needs_header {
                    writeln!(
                        writer,
                        "tick,position_x,position_y,velocity_x,velocity_y,speed,energy_current,energy_max,energy_ratio,age,size,organism_type,behavior_state,state_time,target_x,target_y,target_entity,sensory_range,aggression,boldness"
                    ).expect("Failed to write CSV header");
                    writer.flush().expect("Failed to flush CSV writer");
                }
                
                // Extract target coordinates
                let (target_x, target_y) = if let Some(target_pos) = behavior.target_position {
                    (target_pos.x, target_pos.y)
                } else {
                    (f32::NAN, f32::NAN)
                };
                
                // Write data row
                writeln!(
                    writer,
                    "{},{:.6},{:.6},{:.6},{:.6},{:.6},{:.6},{:.6},{:.6},{},{:.6},{:?},{},{:.6},{:.6},{:.6},{:?},{:.6},{:.6},{:.6}",
                    tick,
                    position.0.x,
                    position.0.y,
                    velocity.0.x,
                    velocity.0.y,
                    speed,
                    energy.current,
                    energy.max,
                    energy.ratio(),
                    age.0,
                    size.value(),
                    org_type,
                    behavior_state,
                    behavior.state_time,
                    target_x,
                    target_y,
                    behavior.target_entity,
                    sensory_range,
                    aggression,
                    boldness
                ).expect("Failed to write CSV row");
                
                writer.flush().expect("Failed to flush CSV writer");
            }
            
            // Mark header as written after dropping writer borrow
            if needs_header {
                tracked_mut.header_written = true;
            }
        } else {
            // Entity no longer exists (probably died)
            info!("[TRACKED] Organism entity {:?} no longer exists", entity);
            tracked_mut.entity = None;
            
            // Close CSV writer
            if let Some(mut writer) = tracked_mut.csv_writer.take() {
                writer.flush().expect("Failed to flush CSV writer on close");
            }
        }
    }
}

