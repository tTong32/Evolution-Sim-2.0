# Cell-Based Plant System Redesign

## Core Architecture Changes
- [ ] Replace individual Producer entities with cell-based plant system
- [ ] Each cell maintains percentage-based plant species distribution (e.g., 60% species 1, 25% species 2, 15% barren)
- [ ] Remove Decomposer entities entirely
- [ ] Add dead organic matter and animal nutrient tracking to cells

## Plant Species System
- [ ] Create PlantSpecies struct with: species_id, genome, percentage, age
- [ ] Store plant species in Cell using SmallVec<[PlantSpecies; 4]> or HashMap for efficiency
- [ ] Each plant species has unique genome determining growth characteristics
- [ ] Plant genome traits:
  - Growth rate (how fast percentage increases)
  - Spread rate (colonization of adjacent cells)
  - Resource efficiency [sunlight, water, mineral] (affects environmental tolerance)
  - Competitive ability (outcompeting other species)
  - Defense mechanisms (poison, thorns - affects consumer behavior)
  - Height/canopy (blocks sunlight for competitors)
  - Root depth (accesses deeper resources)
  - Lifespan (death rate)
- [ ] Each plant species is determined by the system that all other organisms currently use

## Plant Growth & Competition
- [ ] Growth phase: species grow based on percentage * growth_rate * resource_availability * environment_match
- [ ] Environment match derived from resource efficiency traits (not hardcoded)
- [ ] Competition phase: species compete for space; better competitors gain percentage
- [ ] Resource consumption: percentage * efficiency * total_available (shared resources)
- [ ] Competition rules: taller plants block sunlight, deeper roots access water, etc.

## Plant Spreading
- [ ] Spreading phase: species spread to neighbors based on percentage * spread_rate
- [ ] Spreading is colonization (existing species spreads), not speciation
- [ ] Higher percentage species spread more aggressively
- [ ] New species can establish in cells with competition from existing species

## Consumer Eating System (All Types)
- [ ] Implement hard-capped consumption rate for ALL consumers (herbivores AND predators)
- [ ] Add "mouth_size" or "consumption_rate" trait to consumer genomes
- [ ] Predators must spend multiple ticks eating killed prey (not instant consumption)
- [ ] Herbivores consume plant percentage over multiple ticks based on trait
- [ ] Consumption reduces plant percentage in cell
- [ ] Eating state requires minimum duration based on trait

## Nutrient Cycling
- [ ] Plant death converts to dead_organic_matter in cell
- [ ] Animal deaths add nutrients to cell (replaces decomposer system)
- [ ] Dead matter decays over time
- [ ] Nutrients boost plant growth rates in cell

## Initialization
- [ ] Start with founder plant species in initial chunks
- [ ] Founder species have random genomes
- [ ] Initial distribution can be random or based on terrain

## Systems to Modify
- [ ] Update Cell struct with plant_species, dead_organic_matter, animal_nutrients
- [ ] Create plant growth/competition system
- [ ] Create plant spreading system
- [ ] Modify handle_eating to work with plant percentages and implement multi-tick consumption
- [ ] Update handle_death to add nutrients to cells
- [ ] Remove Producer/Decomposer entity spawning
- [ ] Update visualization to only show Consumers (plants shown as cell-level data later)
- [ ] Create plant genome trait expression system (different from animal traits)

---

# Predation & Hunting System Redesign

## Current System Issues
- [ ] Replace abstract "Prey" resource consumption with actual organism killing
- [ ] Currently no individual learning - all organisms of same species identical
- [ ] Currently no pack hunting - only size-based 1v1 predation
- [ ] Currently no species-specific prey preferences
- [ ] Currently no desperation mechanics

## Individual Learning System
- [ ] Create IndividualLearning component to track per-organism prey knowledge
  - `prey_knowledge: HashMap<u32, f32>` - prey_species_id -> huntability_score (0.0-1.0)
  - `learning_rate: f32` - from genome, how fast they learn
  - `teaching_ability: f32` - from genome, how well they pass knowledge to offspring
- [ ] Update learning after each hunt attempt (success increases score, failure decreases)
- [ ] Unknown prey starts at neutral score (0.3-0.5) and evolves through experience

## Pack Hunting System (Family-Based)
- [ ] Add PackTendency traits to genome:
  - `forms_packs: bool` - whether this species forms packs
  - `pack_lifetime: f32` - how long to stay in pack (from genome)
  - `pack_size_preference: usize` - preferred pack size
- [ ] Create Family component to track parent-child relationships
  - `parent: Entity`
  - `children: Vec<Entity>`
  - `pack_id: Option<u32>` - if part of larger pack
  - `formation_time: f32` - how long pack has existed
- [ ] Pack formation: organisms with pack genome form packs through family (parents + children)
- [ ] Pack disbanding: organisms leave pack based on pack_lifetime trait (or stay for life if genome dictates)
- [ ] Pack hunting calculation: multiple predators can take down larger prey
  - Pack size bonus: `(pack_size.sqrt() * coordination)` allows hunting prey up to 2x individual size
  - Solo hunting: still limited by size ratio (1.5x rule)

## Learning Inheritance System
- [ ] Create ParentChildRelationship component to track:
  - `parent: Entity`
  - `child: Entity`
  - `time_together: f32` - cumulative time spent near each other
  - `knowledge_transferred: f32` - how much knowledge has been passed
- [ ] Knowledge transfer system: when parent and child are together, transfer knowledge
  - Transfer rate = `(time_together / teaching_ability).min(1.0)`
  - More time together = more knowledge transferred
  - Teaching ability from genome affects transfer efficiency
- [ ] Update time_together when parent and child are within proximity
- [ ] Inherited knowledge starts at parent's learned values (not neutral)

## Actual Killing System
- [ ] Replace abstract "Prey" resource consumption with actual organism damage/killing
- [ ] When consumer hunts prey organism:
  - Reduce prey's energy based on predator's attack strength
  - If prey energy reaches 0, prey dies (despawn)
  - Predator gains energy from successful kill
  - Update predator's learning (successful hunt increases knowledge of that prey species)
- [ ] Multi-tick eating: predators must spend multiple ticks eating killed prey (based on "mouth_size" or "consumption_rate" trait)
- [ ] Hard-capped consumption rate for all consumers (herbivores AND predators)

## Multi-Factor Predation System
- [ ] Replace size-only predation with multi-factor calculation:
  - Size factor: still matters, but pack can compensate
  - Speed factor: relative speed affects catch success
  - Defense factor: prey defenses (armor, poison) reduce success
  - Pack factor: pack size provides hunting bonus
  - Strategy factor: different hunting strategies (ambush, pursuit, pack) have different success calculations
- [ ] Compensatory model: factors can offset each other (pack size can compensate for size disadvantage)
- [ ] Desperation mechanics:
  - When energy low, expand prey options (accept higher risk)
  - Desperation threshold: try riskier-but-plausible prey, not impossible ones
  - Lower success threshold when desperate (0.3 → 0.1 when starving)

## Prey Selection System
- [ ] Use learned knowledge to select prey targets (not just size)
- [ ] Prey selection algorithm:
  1. Check learned knowledge score for each nearby organism's species
  2. Apply desperation modifier (expand options when desperate)
  3. Check physical constraints (size, speed, defenses)
  4. Calculate risk/reward ratio
  5. Select best option based on learned knowledge + current situation
- [ ] Unknown prey species: start with neutral/moderate score, learn through attempts

## Systems to Modify
- [ ] Modify handle_eating to actually damage/kill prey organisms (not just consume resources)
- [ ] Modify is_prey_of to use learned knowledge + multi-factor calculation (not just size)
- [ ] Modify behavior system to use individual learning for prey selection
- [ ] Add pack detection system (identify nearby family members for pack hunting)
- [ ] Add parent-child relationship tracking system
- [ ] Add learning update system (after each hunt attempt)
- [ ] Add knowledge inheritance system (parent to child transfer)

## Genome Traits to Add
- [ ] Pack hunting traits: `forms_packs`, `pack_lifetime`, `pack_size_preference`, `coordination`
- [ ] Learning traits: `learning_rate`, `teaching_ability`
- [ ] Predation traits: `hunting_strategy` (ambush/pursuit/pack), `attack_strength`, `consumption_rate`
- [ ] Defense traits (for prey): `armor`, `poison_strength`, `flee_speed`, `endurance`

## Notes
- Pack formation is family-based (parents + children), not just proximity
- Learning is individual per organism, not species-wide
- Knowledge inheritance depends on time spent with parents (genome-controlled)
- Desperation expands options but still respects physical constraints
- Multi-factor predation allows pack hunting of larger prey
- Actual killing replaces abstract resource consumption

---

# Offspring Care & Development System

## Spawning System
- [ ] Add `SpawnType` to genome: Egg or Baby (direct spawn)
- [ ] **Initial Stats Based on Spawn Type:**
  - Baby spawn: 10% of max stats (size, energy capacity, etc.)
  - Egg hatch: 30% of max stats (more developed)
- [ ] **Egg System:**
  - Create `Egg` component with:
    - `parent: Entity` (mother, potentially father)
    - `incubation_time_remaining: f32` (from genome)
    - `position: Vec2` (where egg was laid)
    - `incubation_type: IncubationType` (Guarded vs Abandoned)
  - Eggs are immobile (cannot move/avoid predators)
  - Eggs hatch into babies with 30% max stats
- [ ] **Baby System (Direct Spawn):**
  - Spawn as baby organism immediately with 10% max stats
  - Can move immediately (attached to parent)

## Parent-Child Attachment System
- [ ] Add `parental_care_age: f32` to genome (age when child becomes independent)
- [ ] Create `ParentalAttachment` component:
  - `parent: Entity` (mother, or father if genome allows)
  - `child: Entity`
  - `care_until_age: f32` (from genome)
- [ ] **Child is "attached" to parent during childhood:**
  - Child follows parent (position locked to parent with small offset)
  - Parent speed reduced based on speed comparison function (formula to be defined later)
  - Speed penalty function compares child speed vs parent speed
- [ ] **Child eats percentage of parent's meal:**
  - When parent consumes food/energy, child gets `parent_consumed * meal_share_percentage`
  - `meal_share_percentage` from genome (e.g., 0.2 = child gets 20% of parent's meal)
  - Parent receives reduced energy: `parent_energy = parent_consumed * (1.0 - meal_share_percentage)`
- [ ] **Multiple children handling:**
  - If parent has multiple children, implement appropriate sharing (meal distribution, speed penalty stacking, etc.)
  - Details to be determined during implementation

## Child Condition Tracking
- [ ] Child has independent condition tracking even when attached:
  - `child.energy` tracked separately (can starve independently)
  - `child.disease` tracked separately (can get infected independently)
  - `child.growth` tracked separately
- [ ] **Child can die even when attached:**
  - If child energy < death threshold → child dies (parent continues)
  - If child gets disease → child can die from disease (parent continues)
  - If child doesn't get enough food → child dies (starvation)
- [ ] **Child communication to parent:**
  - Mother "just knows" child's condition (parent can query child state directly)
  - No explicit communication system needed - parent reads child's energy/disease/growth state
  - Parent behavior modified based on child needs:
    - If child low energy → parent prioritizes hunting/eating
    - If child diseased → parent might avoid dangerous areas

## Knowledge Transfer System (Continuous)
- [ ] Continuous transfer during childhood (Option C)
- [ ] Knowledge gradually accumulates as parent and child spend time together
- [ ] Transfer mechanics to be defined later (rate, amount, frequency)
- [ ] More time together = more knowledge accumulated
- [ ] If child abandoned early: knowledge transfer stops, child keeps what they have

## Growth System
- [ ] Add `growth_rate: f32` to genome (base growth rate per day)
- [ ] Add `max_growth_rate: f32` to genome (maximum growth rate cap)
- [ ] Growth calculation:
  - Base growth: `growth_rate` per day
  - Food bonus: additional growth from food consumed (capped by `max_growth_rate`)
  - Total growth: `min(base_growth + food_bonus, max_growth_rate)`
- [ ] **Starvation handling:**
  - If child doesn't get enough food → child dies (no growth without food)
  - Minimum food requirement for survival (from genome or fixed threshold)
  - Track `food_deficit: f32` - if exceeds threshold, child dies

## Independence System (Age-Based)
- [ ] Independence determined solely by age (for simplicity)
- [ ] Add `independence_age: f32` to genome (age when child becomes independent)
- [ ] Check each tick: if `child.age >= independence_age` → independence
- [ ] At independence:
  - Final knowledge transfer (accumulated knowledge finalized)
  - Parent-child attachment removed
  - Child can move independently
  - Child must survive on own

## Parental Death Scenarios
- [ ] **If parent hunted by predator:**
  - Both parent and child die (child attached, cannot escape)
  - No escape mechanism - child always dies with parent if parent is hunted
- [ ] **If parent dies from other causes (disease, starvation, etc.):**
  - Child survives but forced independence
  - Child keeps accumulated knowledge (up to death point)
  - Child must attempt to survive alone
  - Child's condition (energy, disease) continues from current state

## Milk Transfer System (Early Stage)
- [ ] **Milk transfer timing:**
  - Active during first 30% of care duration (early stage)
  - Or active until child reaches 20% growth (infant stage)
  - Whichever comes first
- [ ] **Milk mechanics:**
  - Only mother can produce milk (father cannot)
  - Requires `can_produce_milk: bool` in genome
  - Direct energy transfer: `child.energy += milk_amount`, `mother.energy -= milk_amount`
  - Milk amount from genome (e.g., 5 energy per tick)
  - Only if mother has sufficient energy

## Father Care System
- [ ] Father can do same things as mother (if genome allows):
  - Can be attachment parent (child follows father)
  - Can share meals with child
  - Can protect child
  - Cannot produce milk (only mother)
- [ ] **When both parents present:**
  - Both can help care for child
  - Child attaches to mother (preference)
  - Father provides backup care (can also share meals, protect)
- [ ] **Parent separation:**
  - If parents separate, baby goes to mother (almost always)
  - Father care ends if parents separate
  - Mother continues care alone
- [ ] Add `father_provides_care: bool` to genome
- [ ] If mother dies but father present: child can attach to father (if genome allows)

## Failure Cases
- [ ] Parent dies during care → child forced independence
- [ ] Parent hunted → both parent and child die (no escape)
- [ ] Child starves → child dies (parent continues)
- [ ] Child gets disease → child can die (parent continues)
- [ ] Child abandoned early → child must survive with accumulated knowledge
- [ ] Multiple children → implement appropriate handling (meal sharing, penalties, etc.)

## Systems to Create
- [ ] Parent-child attachment system (child follows parent, speed penalty - formula to be defined)
- [ ] Meal sharing system (child gets percentage of parent's meal)
- [ ] Child condition tracking (independent energy, disease, growth)
- [ ] Parent awareness system (parent reads child state directly, no explicit communication needed)
- [ ] Continuous knowledge transfer system (mechanics to be defined later)
- [ ] Growth system (base rate + food bonus, capped, starvation death)
- [ ] Independence check system (age-based)
- [ ] Parental death handling (hunted = both die, other causes = child survives)
- [ ] Milk transfer system (early stage, mother only)
- [ ] Father care system (can attach, share meals, but no milk; both parents can help when present)
- [ ] Parent separation handling (baby goes to mother)
- [ ] Multiple children handling system

## Genome Traits Needed
- [ ] `spawn_type: SpawnType` - Egg or Baby
- [ ] `incubation_type: IncubationType` - Guarded or Abandoned (if egg)
- [ ] `incubation_duration: f32` - How long eggs incubate
- [ ] `parental_care_age: f32` - Age when child becomes independent
- [ ] `meal_share_percentage: f32` - Percentage of parent's meal child gets (e.g., 0.2)
- [ ] `growth_rate: f32` - Base growth rate per day
- [ ] `max_growth_rate: f32` - Maximum growth rate cap
- [ ] `father_provides_care: bool` - Whether father helps
- [ ] `can_produce_milk: bool` - Whether mother can transfer energy (milk)
- [ ] `milk_amount: f32` - Energy transferred per tick (if milk enabled)
- [ ] `knowledge_transfer_rate: f32` - How fast knowledge transfers to child (to be defined later)

## Integration Points
- [ ] Reproduction system: spawn as Egg or Baby with appropriate initial stats (10% vs 30%)
- [ ] Energy system: meal sharing, milk transfer, child starvation
- [ ] Behavior system: parent reads child state and adjusts behavior (prioritizes food if child hungry)
- [ ] Movement system: child attached to parent, speed penalty calculation (formula to be defined)
- [ ] Learning system: continuous knowledge transfer during childhood (mechanics to be defined)
- [ ] Disease system: child can get infected independently, can die from disease

## Notes
- Child attached to parent = follows parent, slows parent down (speed penalty formula to be defined)
- Child eats percentage of parent's meal (genome-defined)
- If parent hunted → both die; if parent dies other causes → child survives alone
- Child condition tracked independently (energy, disease, growth)
- Parent "just knows" child's condition (queries child state directly, no explicit communication system)
- Knowledge transfers continuously during childhood (mechanics to be defined later)
- Growth has max cap, child dies without enough food
- Independence based on age only (simplicity)
- Father can care (except milk), both parents can help when present, baby goes to mother if parents separate
- Milk active in early stage (first 30% of care or until 20% growth)
- Multiple children handling to be implemented appropriately
- No escape mechanism - child always dies if parent is hunted

---

# Visualization Rework
