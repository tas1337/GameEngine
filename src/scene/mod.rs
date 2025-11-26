// Scene management - built from scratch!

use crate::ecs::*;
// use crate::renderer::*;  // Not currently used
use crate::core::Engine;

pub struct Scene {
    pub entities: EntityManager,
}

impl Scene {
    pub fn new(capacity: usize) -> Self {
        Self {
            entities: EntityManager::new(capacity),
        }
    }

    pub fn update(&mut self, engine: &Engine) {
        // Update physics
        self.entities.update_physics(engine.delta_time);
        
        // Update lifetimes
        self.entities.update_lifetimes(engine.delta_time);
    }

    pub fn clear(&mut self) {
        // Remove all entities by destroying them one by one
        let entities: Vec<_> = self.entities.entities.clone();
        for entity in entities {
            self.entities.destroy_entity(entity);
        }
    }
}

