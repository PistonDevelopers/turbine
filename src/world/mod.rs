//! World ECS structure.

use math::Vec3;
use math::AABB;

/// The maximum number of entities.
pub const ENTITY_COUNT: usize = 10000;

bitflags!(
    /// Used to turn on/off components per entity.
    flags Mask: u32 {
        /// Entity is alive.
        const ALIVE         = 0b00000001,
        /// Entity is selected.
        const SELECT        = 0b00000010,
        /// Entity has an AABB.
        const AABB          = 0b00000100,
    }
);

/// Stores physical state.
pub struct Physics {
    /// The position.
    pub position: Vec<Vec3>,
}

impl Physics {
    /// Returns new `Physics`.
    pub fn new() -> Physics {
        Physics {
            position: vec![[0.0; 3]; ENTITY_COUNT],
        }
    }

    /// Gets next linear step.
    pub fn step(&mut self, prev: &Physics, current: &Physics) {
        use math::Vector;

        for i in 0..ENTITY_COUNT {
            // current + (current - prev) = 2 * current - prev.
            self.position[i] = current.position[i]
                .scale(2.0)
                .sub(prev.position[i]);
        }
    }
}

/// Stores the world data.
pub struct World {
    /// The active components per entity.
    pub mask: Vec<Mask>,
    /// The initial state of physics.
    pub init: Physics,
    /// The previous state.
    pub prev: Physics,
    /// The current state.
    pub current: Physics,
    /// The next state.
    pub next: Physics,
    /// An AABB relative to position.
    pub aabb: Vec<AABB>,
}

impl World {
    /// Returns a new `World`.
    pub fn new() -> World {
        World {
            mask: vec![Mask::empty(); ENTITY_COUNT],
            init: Physics::new(),
            prev: Physics::new(),
            current: Physics::new(),
            next: Physics::new(),
            aabb: vec![AABB::empty(); ENTITY_COUNT]
        }
    }

    /// Swaps the physical state such that previous is now next.
    pub fn swap_physics(&mut self) {
        use std::mem::swap;

        swap(&mut self.prev, &mut self.current);
        swap(&mut self.current, &mut self.next);
    }

    /// Finds the first free entity slot.
    pub fn find_free_entity_slot(&self) -> Option<usize> {
        for i in 0..ENTITY_COUNT {
            if self.mask[i].is_empty() { return Some(i); }
        }
        None
    }

    /// Sets position with no velocity.
    pub fn set_position_with_no_velocity(&mut self, id: usize, pos: Vec3) {
        self.prev.position[id] = pos;
        self.current.position[id] = pos;
        self.next.position[id] = pos;
    }

    /// Adds a new entity.
    /// Marks the entity as alive and selects it.
    pub fn add_entity(&mut self, pos: Vec3) {
        let id = match self.find_free_entity_slot() {
            Some(id) => id,
            None => {
                warn!("There are no free entity slots");
                return;
            }
        };
        self.init.position[id] = pos;
        self.set_position_with_no_velocity(id, pos);
        let mask = &mut self.mask[id];
        mask.insert(ALIVE);
        mask.insert(SELECT);
        info!("Added entity id {}", id);
    }
}
