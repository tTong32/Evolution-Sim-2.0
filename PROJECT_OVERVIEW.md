# 🧬 Evolutionary Ecosystem Simulator — Project Overview

A modular, open-ended simulation of biological evolution and ecosystems featuring dynamic environments, emergent behaviors, and millions of autonomous agents.  
This document summarizes all major systems, dependencies, and development milestones.

---

## 🧱 Core Vision

- **Goal:** Simulate natural selection and ecological interactions in a scalable virtual world.  
- **Scale:** 100,000 – 1,000,000 agents across a 2D world divided into evolving biomes.  
- **Language Stack:**  
  - **Backend:** Rust (performance, memory safety, parallelism)  
  - **Frontend / Visualization:** Rust+Bevy (for interactive visualization)  
  - **Data:** CSV / JSON logging for analytics  

---

## ⚙️ System Overview

### 1. **World System**
- **Structure:**  
  - 2D grid divided into *cells* (chunks), each tracking temperature, humidity, terrain type, elevation, resource density, etc.  
- **Features:**  
  - Dynamic climate simulation (temperature gradients, rainfall, weather cycles).  
  - Events: volcanic eruptions, droughts, floods, meteor strikes.  
  - Seasonal and long-term change (ice ages, global warming).  
- **Performance:** Use sparse chunks & parallel updates (Rust multithreading).

---

### 2. **Resource System**
- **Types:** Plant matter, minerals, sunlight, water, detritus, prey biomass.  
- **Properties:**  
  - Regeneration rate, decay rate, nutrient value.  
  - Resources evolve and mutate (e.g. plants adapt leaf shape or toxin production).  
- **Flow:**  
  - Resources flow between neighboring cells (e.g. nutrient runoff, seed dispersal).  

---

### 3. **Organism System**
- **Entities:** Every living thing (animal, plant, fungus, microbe).  
- **Attributes:** Position, velocity, energy, age, size, color, sensory organs, reproduction cooldown, genome.  
- **Lifecycle:** Spawn → Grow → Reproduce → Die → Decompose.  
- **Interactions:** Predation, competition, symbiosis, mating, territorial behavior.  
- **Scaling:** Efficient entity-component-system (ECS) design (Bevy ECS or custom Rust ECS).

---

### 4. **Genetic System**
- **Representation:**  
  - Each organism has a *genome* (array of floating-point genes).  
  - Each gene encodes physical and behavioral traits.  
- **Mutation & Crossover:**  
  - Random mutations with Gaussian noise.  
  - Sexual reproduction mixes parental genes.  
- **Expression:**  
  - Genes map to traits via configurable trait functions (e.g. eye_sensitivity = sigmoid(gene[2])).  
  - Enables emergent traits like camouflage, speed, metabolism without hardcoding behaviors.

---

### 5. **Behavior System**
- **Goal:** Simulate adaptive, non-AI decision-making.  
- **Mechanism:**  
  - Each tick, organisms use local sensory data (sight, smell, energy) to pick an action (move, eat, mate, flee).  
  - Rule-based or small state-machine approach for scalability.  
  - Behavior affected by genetic parameters (e.g. “boldness”, “curiosity”, “aggression”).  
- **Actions:** Wander, chase, forage, rest, reproduce, migrate.

---

### 6. **Metabolism & Energy System**
- **Energy Flow:**  
  - Organisms consume food/resources → convert to energy → expend on movement, growth, reproduction.  
  - Starvation or inefficiency → death → nutrients recycled into environment.  
- **Evolvable metabolism:** Genes can change digestion rate, energy efficiency, toxin resistance.

---

### 7. **Reproduction & Heredity**
- **Types:** Asexual (clone + mutation), sexual (gene crossover + mutation).  
- **Parent Selection:** Fitness-weighted random choice.  
- **Offspring Generation:**  
  - Offspring inherits genome from parent(s).  
  - Mutation rate and genome length are evolvable.

---

### 8. **Ecosystem & Emergent Dynamics**
- Natural trophic layers (producers → consumers → decomposers).  
- Biomes emerge (forests, deserts, tundra) via resource distribution + climate.  
- Population dynamics: extinction, radiation, co-evolution, speciation.  
- Environmental feedback loops (e.g., more plants → more herbivores → more predators).  

---

### 9. **Statistics, Logging & Visualization**
- Real-time data collection (population counts, mean traits, mutation frequency).  
- Heatmaps for resource density and organism distribution.  
- Time-series graphs for ecosystem health and diversity.  
- Visualization modes:  
  - Overview map (biomes, resources, organisms).  
  - Trait evolution charts over time.  

---

### 10. **Event & Climate System**
- **Events:** Volcanoes, floods, meteor strikes, disease outbreaks.  
- **Climate:** Dynamic temperature + humidity cycles (seasons, long-term drift).  
- **Impact:** Alters resource regrowth, mortality, migration, adaptation pressure.  

---

## 🧩 Development Timeline (Suggested Order)

| Stage | System | Description |
|-------|---------|-------------|
| 1 | **Core Framework** | Create project structure, ECS framework, world grid. |
| 2 | **World & Resource Simulation** | Implement terrain, resources, and climate updates. |
| 3 | **Organisms (Basic)** | Add agents with position, energy, metabolism, simple behavior. |
| 4 | **Genetics & Reproduction** | Add genome encoding, mutation, crossover. |
| 5 | **Behavior System** | Implement decision rules (wander, eat, flee, mate). |
| 6 | **Resource-Organism Interaction** | Link eating/metabolism with resource map. |
| 7 | **Visualization & Logging** | Add real-time data collection and map visualization. |
| 8 | **Emergent Ecosystem Tuning** | Tune rates until emergent biomes and dynamics form. |
| 9 | **Advanced Systems** | Add speciation, climate events, disease, co-evolution. |
| 10 | **Performance Scaling** | Parallelize updates, optimize data layout, add chunking. |

---

## 🧠 Design Notes

- **ECS architecture** ensures modularity and parallelism.  
- **Stateless updates** enable parallel chunk computation.  
- **Rust + Bevy ECS** provides scalability to millions of agents.  
- **Open-ended evolution** emerges from fitness landscape + resource feedback loops.  
- **Visualization** handled by separate async process for performance.

---

## 📊 Future Extensions

- Genetic drift and speciation tracking.  
- Climate-driven migration and extinction events.  
- Procedural terrain and biome generation.  
- Trophic network visualizer.  
- AI-assisted ecosystem summarizer (auto-analyzes evolutionary trends).  

---

## 🧩 Repository Layout (Planned)
evo_sim/
├── Cargo.toml
├── src/
│   ├── main.rs 
│   ├── world/
│   ├── resources/
│   ├── organisms/
│   ├── genetics/
│   ├── behavior/
│   ├── metabolism/
│   ├── visualization/
│   └── utils/
├── data/
│   ├── logs/
│   ├── configs/
│   └── outputs/
└── docs/
└── PROJECT_OVERVIEW.md


---

**Author’s Note:**  
This project is designed for scientific and creative exploration — to watch complexity and life emerge from simple, local rules.


