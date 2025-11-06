use bevy::prelude::*;
use crate::organisms::components::*;
use crate::organisms::genetics::{Genome, traits, DEFAULT_MUTATION_RATE};
use crate::world::WorldGrid;
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

/// Update organism movement (simple wandering behavior)
/// Uses speed trait from genome if available
pub fn update_movement(
    mut query: Query<(&mut Position, &mut Velocity, &Energy, &Size, Option<&Genome>, Entity)>,
    time: Res<Time>,
    tracked: ResMut<TrackedOrganism>,
) {
    let dt = time.delta_seconds();
    let mut rng = rand::thread_rng();
    let mut direction_changed = false;
    
    for (mut position, mut velocity, energy, size, genome_opt, entity) in query.iter_mut() {
        // Skip if dead or very low energy
        if energy.is_dead() || energy.ratio() < 0.1 {
            velocity.0 = Vec2::ZERO;
            if tracked.entity == Some(entity) {
                info!("[TRACKED] Organism stopped moving (low energy: {:.2}%)", energy.ratio() * 100.0);
            }
            continue;
        }
        
        // Get max speed from genome or default based on size
        let max_speed = if let Some(genome) = genome_opt {
            traits::express_speed(genome)
        } else {
            // Fallback: larger organisms move slower
            20.0 / size.value().max(0.5)
        };
        
        // Simple wandering behavior: random walk with momentum
        // Occasionally add random velocity changes
        if rng.gen_bool(0.05) { // 5% chance per frame to change direction
            let angle = rng.gen_range(0.0..std::f32::consts::TAU);
            let speed = rng.gen_range(max_speed * 0.3..max_speed * 0.7);
            velocity.0 = Vec2::from_angle(angle) * speed;
            
            if tracked.entity == Some(entity) {
                direction_changed = true;
            }
        }
        
        // Apply velocity damping (friction)
        velocity.0 *= 0.95;
        
        // Clamp velocity to max speed
        if velocity.0.length() > max_speed {
            velocity.0 = velocity.0.normalize() * max_speed;
        }
        
        // Update position
        let old_position = position.0;
        position.0 += velocity.0 * dt;
        
        // Simple boundary checking (keep organisms within reasonable bounds)
        // In future, this will wrap or use proper world boundaries
        let max_pos = 200.0;
        let hit_boundary = position.0.x != old_position.x + velocity.0.x * dt || 
                          position.0.y != old_position.y + velocity.0.y * dt;
        position.0.x = position.0.x.clamp(-max_pos, max_pos);
        position.0.y = position.0.y.clamp(-max_pos, max_pos);
        
        if tracked.entity == Some(entity) && hit_boundary {
            info!("[TRACKED] Organism hit world boundary at ({:.2}, {:.2})", position.0.x, position.0.y);
        }
    }
    
    // Log direction change if it happened
    if direction_changed {
        if let Some(entity) = tracked.entity {
            if let Ok((_position, velocity, _energy, _size, _genome, _entity)) = query.get(entity) {
                info!("[TRACKED] Direction changed - New velocity: ({:.2}, {:.2}), Speed: {:.2}", 
                      velocity.0.x, velocity.0.y, velocity.0.length());
            }
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
    query: Query<(Entity, &Position, &Velocity, &Energy, &Age, &Size, &OrganismType), With<Alive>>,
) {
    let mut tracked_mut = tracked;
    tracked_mut.log_counter += 1;
    
    // Log every 10 ticks for more frequent output (change to 60 for less frequent)
    if tracked_mut.log_counter % 10 != 0 {
        return;
    }
    
    if let Some(entity) = tracked_mut.entity {
        if let Ok((_entity, position, velocity, energy, age, size, org_type)) = query.get(entity) {
            let speed = velocity.0.length();
            let action = if speed < 0.1 {
                "Resting"
            } else if speed > 10.0 {
                "Moving Fast"
            } else {
                "Wandering"
            };
            
            // Console logging
            info!(
                "[TRACKED ORGANISM] Tick: {} | Pos: ({:.2}, {:.2}) | Vel: ({:.2}, {:.2}) | Speed: {:.2} | Energy: {:.2}/{:.2} ({:.1}%) | Age: {} | Size: {:.2} | Type: {:?} | Action: {}",
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
                action
            );
            
            // CSV logging
            let needs_header = !tracked_mut.header_written;
            let tick = tracked_mut.log_counter;
            
            if let Some(ref mut writer) = tracked_mut.csv_writer {
                // Write CSV header if not written yet
                if needs_header {
                    writeln!(
                        writer,
                        "tick,position_x,position_y,velocity_x,velocity_y,speed,energy_current,energy_max,energy_ratio,age,size,organism_type,action"
                    ).expect("Failed to write CSV header");
                    writer.flush().expect("Failed to flush CSV writer");
                }
                
                // Write data row
                writeln!(
                    writer,
                    "{},{:.6},{:.6},{:.6},{:.6},{:.6},{:.6},{:.6},{:.6},{},{:.6},{:?},{}",
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
                    action
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

