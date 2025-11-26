// Entity system - built from scratch!

use crate::core::memory::{Handle, GenerationalPool};
use super::component::*;

/// Entity - just an ID
pub type Entity = Handle;

/// Component storage for all entities
pub struct EntityManager {
    // Component pools
    pub transforms: GenerationalPool<Transform>,
    pub velocities: GenerationalPool<Velocity>,
    pub colors: GenerationalPool<Color>,
    pub lifetimes: GenerationalPool<Lifetime>,
    
    // Active entities
    pub entities: Vec<Entity>,
}

impl EntityManager {
    pub fn new(capacity: usize) -> Self {
        Self {
            transforms: GenerationalPool::new(capacity),
            velocities: GenerationalPool::new(capacity),
            colors: GenerationalPool::new(capacity),
            lifetimes: GenerationalPool::new(capacity),
            entities: Vec::new(),
        }
    }

    /// Create a new entity with a transform
    pub fn create_entity(&mut self, transform: Transform) -> Option<Entity> {
        if let Some(handle) = self.transforms.alloc(transform) {
            self.entities.push(handle);
            Some(handle)
        } else {
            None
        }
    }

    /// Create a particle entity (with all components)
    pub fn create_particle(
        &mut self,
        transform: Transform,
        velocity: Velocity,
        color: Color,
        lifetime: Lifetime,
    ) -> Option<Entity> {
        if let Some(entity) = self.create_entity(transform) {
            // Add other components
            self.velocities.alloc(velocity);
            self.colors.alloc(color);
            self.lifetimes.alloc(lifetime);
            Some(entity)
        } else {
            None
        }
    }

    /// Destroy an entity and its components
    pub fn destroy_entity(&mut self, entity: Entity) {
        self.transforms.free(entity);
        self.velocities.free(entity);
        self.colors.free(entity);
        self.lifetimes.free(entity);
        
        // Remove from active entities
        if let Some(index) = self.entities.iter().position(|&e| e == entity) {
            self.entities.swap_remove(index);
        }
    }

    /// Update physics for entities with velocity
    pub fn update_physics(&mut self, delta_time: f32) {
        for &entity in &self.entities {
            if let (Some(transform), Some(velocity)) = (
                self.transforms.get_mut(entity),
                self.velocities.get(entity),
            ) {
                transform.position += velocity.linear * delta_time;
            }
        }
    }

    /// Update lifetimes and remove dead particles
    pub fn update_lifetimes(&mut self, delta_time: f32) {
        let mut dead_entities = Vec::new();

        for &entity in &self.entities {
            if let Some(lifetime) = self.lifetimes.get_mut(entity) {
                lifetime.remaining -= delta_time;
                if !lifetime.is_alive() {
                    dead_entities.push(entity);
                }
            }
        }

        // Remove dead entities
        for entity in dead_entities {
            self.destroy_entity(entity);
        }
    }

    pub fn entity_count(&self) -> usize {
        self.entities.len()
    }
}

