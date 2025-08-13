//! # Example Rust File for Xylux IDE
//!
//! This file demonstrates various Rust features that the specialized tools can analyze

use std::collections::HashMap;
use std::fs::File;
use std::io::{self, Read, Write};
use std::path::Path;

/// A simple struct to demonstrate Rust features
#[derive(Debug, Clone)]
pub struct GameEntity {
    pub id: u32,
    pub name: String,
    pub position: (f32, f32, f32),
    pub health: f32,
    pub is_active: bool,
}

/// Trait for objects that can be updated
pub trait Updatable {
    fn update(&mut self, delta_time: f32);
}

/// Implementation of Updatable for GameEntity
impl Updatable for GameEntity {
    fn update(&mut self, delta_time: f32) {
        if self.is_active {
            // Simple movement simulation
            self.position.0 += delta_time * 10.0;

            // Health regeneration
            if self.health < 100.0 {
                self.health = (self.health + delta_time * 5.0).min(100.0);
            }
        }
    }
}

/// Game world manager
pub struct GameWorld {
    entities: HashMap<u32, GameEntity>,
    next_id: u32,
}

impl GameWorld {
    /// Create a new game world
    pub fn new() -> Self {
        Self { entities: HashMap::new(), next_id: 1 }
    }

    /// Add a new entity to the world
    pub fn add_entity(&mut self, name: String, position: (f32, f32, f32)) -> u32 {
        let id = self.next_id;
        self.next_id += 1;

        let entity = GameEntity { id, name, position, health: 100.0, is_active: true };

        self.entities.insert(id, entity);
        id
    }

    /// Remove an entity from the world
    pub fn remove_entity(&mut self, id: u32) -> Option<GameEntity> {
        self.entities.remove(&id)
    }

    /// Get a reference to an entity
    pub fn get_entity(&self, id: u32) -> Option<&GameEntity> {
        self.entities.get(&id)
    }

    /// Get a mutable reference to an entity
    pub fn get_entity_mut(&mut self, id: u32) -> Option<&mut GameEntity> {
        self.entities.get_mut(&id)
    }

    /// Update all entities in the world
    pub fn update(&mut self, delta_time: f32) {
        for entity in self.entities.values_mut() {
            entity.update(delta_time);
        }
    }

    /// Get the number of active entities
    pub fn active_entity_count(&self) -> usize {
        self.entities.values().filter(|e| e.is_active).count()
    }

    /// Save world state to file
    pub fn save_to_file<P: AsRef<Path>>(&self, path: P) -> io::Result<()> {
        let mut file = File::create(path)?;

        for entity in self.entities.values() {
            writeln!(
                file,
                "{},{},{},{},{},{},{}",
                entity.id,
                entity.name,
                entity.position.0,
                entity.position.1,
                entity.position.2,
                entity.health,
                entity.is_active
            )?;
        }

        Ok(())
    }

    /// Load world state from file
    pub fn load_from_file<P: AsRef<Path>>(&mut self, path: P) -> io::Result<()> {
        let mut file = File::open(path)?;
        let mut contents = String::new();
        file.read_to_string(&mut contents)?;

        self.entities.clear();
        let mut max_id = 0;

        for line in contents.lines() {
            let parts: Vec<&str> = line.split(',').collect();
            if parts.len() == 7 {
                let id: u32 = parts[0].parse().unwrap_or(0);
                let name = parts[1].to_string();
                let x: f32 = parts[2].parse().unwrap_or(0.0);
                let y: f32 = parts[3].parse().unwrap_or(0.0);
                let z: f32 = parts[4].parse().unwrap_or(0.0);
                let health: f32 = parts[5].parse().unwrap_or(100.0);
                let is_active: bool = parts[6].parse().unwrap_or(true);

                let entity = GameEntity { id, name, position: (x, y, z), health, is_active };

                self.entities.insert(id, entity);
                max_id = max_id.max(id);
            }
        }

        self.next_id = max_id + 1;
        Ok(())
    }
}

impl Default for GameWorld {
    fn default() -> Self {
        Self::new()
    }
}

/// Game configuration
#[derive(Debug, Clone)]
pub struct GameConfig {
    pub window_width: u32,
    pub window_height: u32,
    pub fullscreen: bool,
    pub vsync: bool,
    pub max_entities: usize,
    pub target_fps: u32,
}

impl Default for GameConfig {
    fn default() -> Self {
        Self {
            window_width: 1920,
            window_height: 1080,
            fullscreen: false,
            vsync: true,
            max_entities: 1000,
            target_fps: 60,
        }
    }
}

/// Main game structure
pub struct Game {
    world: GameWorld,
    config: GameConfig,
    running: bool,
    last_update: std::time::Instant,
}

impl Game {
    /// Create a new game instance
    pub fn new(config: GameConfig) -> Self {
        Self {
            world: GameWorld::new(),
            config,
            running: false,
            last_update: std::time::Instant::now(),
        }
    }

    /// Start the game
    pub fn start(&mut self) {
        self.running = true;
        self.last_update = std::time::Instant::now();

        // Add some example entities
        self.world.add_entity("Player".to_string(), (0.0, 0.0, 0.0));
        self.world.add_entity("Enemy1".to_string(), (10.0, 0.0, 5.0));
        self.world.add_entity("Enemy2".to_string(), (-5.0, 0.0, 10.0));
    }

    /// Stop the game
    pub fn stop(&mut self) {
        self.running = false;
    }

    /// Check if the game is running
    pub fn is_running(&self) -> bool {
        self.running
    }

    /// Game main loop update
    pub fn update(&mut self) {
        if !self.running {
            return;
        }

        let now = std::time::Instant::now();
        let delta_time = now.duration_since(self.last_update).as_secs_f32();
        self.last_update = now;

        // Update world
        self.world.update(delta_time);

        // Check win/lose conditions
        if self.world.active_entity_count() <= 1 {
            println!("Game Over! Only {} entities remaining", self.world.active_entity_count());
        }
    }

    /// Get reference to the game world
    pub fn world(&self) -> &GameWorld {
        &self.world
    }

    /// Get mutable reference to the game world
    pub fn world_mut(&mut self) -> &mut GameWorld {
        &mut self.world
    }

    /// Get game configuration
    pub fn config(&self) -> &GameConfig {
        &self.config
    }
}

// Example functions with various complexity levels
pub fn calculate_distance(pos1: (f32, f32, f32), pos2: (f32, f32, f32)) -> f32 {
    let dx = pos1.0 - pos2.0;
    let dy = pos1.1 - pos2.1;
    let dz = pos1.2 - pos2.2;
    (dx * dx + dy * dy + dz * dz).sqrt()
}

pub fn normalize_vector(v: (f32, f32, f32)) -> (f32, f32, f32) {
    let magnitude = (v.0 * v.0 + v.1 * v.1 + v.2 * v.2).sqrt();
    if magnitude > 0.0 {
        (v.0 / magnitude, v.1 / magnitude, v.2 / magnitude)
    } else {
        (0.0, 0.0, 0.0)
    }
}

/// Example macro for debugging
macro_rules! debug_entity {
    ($entity:expr) => {
        println!(
            "Entity {}: {} at {:?} (HP: {})",
            $entity.id, $entity.name, $entity.position, $entity.health
        );
    };
}

/// Example async function
pub async fn load_game_data(path: &str) -> Result<Vec<u8>, Box<dyn std::error::Error>> {
    use tokio::fs::File;
    use tokio::io::AsyncReadExt;

    let mut file = File::open(path).await?;
    let mut buffer = Vec::new();
    file.read_to_end(&mut buffer).await?;
    Ok(buffer)
}

/// Example tests
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_game_world_creation() {
        let world = GameWorld::new();
        assert_eq!(world.active_entity_count(), 0);
    }

    #[test]
    fn test_entity_addition() {
        let mut world = GameWorld::new();
        let id = world.add_entity("Test Entity".to_string(), (0.0, 0.0, 0.0));
        assert_eq!(world.active_entity_count(), 1);
        assert!(world.get_entity(id).is_some());
    }

    #[test]
    fn test_distance_calculation() {
        let pos1 = (0.0, 0.0, 0.0);
        let pos2 = (3.0, 4.0, 0.0);
        let distance = calculate_distance(pos1, pos2);
        assert!((distance - 5.0).abs() < f32::EPSILON);
    }

    #[test]
    fn test_vector_normalization() {
        let v = (3.0, 4.0, 0.0);
        let normalized = normalize_vector(v);
        let magnitude = (normalized.0 * normalized.0
            + normalized.1 * normalized.1
            + normalized.2 * normalized.2)
            .sqrt();
        assert!((magnitude - 1.0).abs() < f32::EPSILON);
    }
}

/// Example main function for testing
#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let config = GameConfig::default();
    let mut game = Game::new(config);

    game.start();

    // Game loop simulation
    for _ in 0..10 {
        game.update();
        tokio::time::sleep(tokio::time::Duration::from_millis(16)).await; // ~60 FPS
    }

    // Save game state
    game.world().save_to_file("game_save.csv")?;

    println!("Game simulation completed!");
    println!("Active entities: {}", game.world().active_entity_count());

    Ok(())
}
