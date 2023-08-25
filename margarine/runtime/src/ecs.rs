use crate::{Data, Stack};

#[derive(Debug)]
pub struct World {
    entities: Vec<Entity>,
    components: Vec<Vec<Component>>,
    systems: Vec<System>,
}


impl World {
    pub fn new() -> Self {
        Self {
            entities: vec![],
            systems: vec![],
            components: vec![],
        }
    }

    
    pub fn new_entity(&mut self) -> EntityId {
        // PERFORMANCE: Switch to a free-list approach
        if let Some((index, entity)) = self.entities.iter_mut().enumerate().find(|x| !x.1.is_in_use) {
            entity.kill_generation();
            entity.is_in_use = true;

            for comp in self.components.iter_mut() {
                comp[index] = Component::empty();
            }
            
            return EntityId { index, generation: entity.generation };
        }


        let mut entity = Entity::new();
        entity.is_in_use = true;
        self.entities.push(entity);
        let index = self.entities.len() - 1;

        for comp in self.components.iter_mut() {
            comp.push(Component::empty());
        }

        EntityId { index, generation: 0 }
    }


    pub fn remove_entity(&mut self, entity_id: EntityId) -> bool {
        let Some(entity) = self.entity_mut(entity_id) else { return false };
        entity.is_in_use = false;
        true
    }


    pub fn entity(&self, entity_id: EntityId) -> Option<&Entity> {
        let entity = self.entities.get(entity_id.index)?;
        if entity.generation != entity_id.generation {
            return None
        }

        Some(entity)
    }


    pub fn entity_mut(&mut self, entity_id: EntityId) -> Option<&mut Entity> {
        let entity = self.entities.get_mut(entity_id.index)?;
        if entity.generation != entity_id.generation {
            return None
        }

        Some(entity)
    }


    pub fn new_component(&mut self) -> ComponentIndex {
        let index = self.components.len();
        let vec = vec![Component::empty(); self.entities.len()];
        self.components.push(vec);
        
        ComponentIndex(index)
    }


    pub fn add_system(&mut self, system: System) {
        self.systems.push(system);
    }


    pub fn set_component_of_entity(&mut self, entity_id: EntityId, component: ComponentIndex, data: Data) -> Option<()> {
        self.entity(entity_id)?;

        let component = self.components.get_mut(component.0)?.get_mut(entity_id.index)?;
        component.is_init = true;
        component.data = data;

        Some(())
    }
}


impl World {
    pub fn run_systems(&mut self, stack: &mut Stack) {
        let mut storage = vec![];
        let mut entity_list = Vec::with_capacity(self.entities.len());
        for s in self.systems.iter() {
            entity_list.clear();
            storage.clear();
            storage.reserve(s.components.len() * self.entities.len());

            'e: for (i, e) in self.entities.iter().enumerate() {
                if !e.is_in_use { continue }

                for c in s.components.iter() {
                    if !self.components[c.0][i].is_init { continue 'e }
                }

                entity_list.push(i);

                
                for c in s.components.iter() {
                    storage.push(self.components[c.0][i].data.clone())
                }
            }


            for e in entity_list.iter() {
                stack.push(self.components.len() + 1);
                for i in (0..s.components.len()).rev() {
                    stack.set_reg(i as u8+1, storage.pop().unwrap())
                }


                // TODO: run code
                stack.set_reg(1, Data::new_bool(true));


                for i in s.components.iter().enumerate().rev() {
                    self.components[i.1.0][*e].data = stack.reg(i.0 as u8 + 1).clone();
                }
                stack.pop(self.components.len() + 1);
            }
        }
    }
}


#[derive(Debug)]
pub struct Entity {
    generation: usize,
    is_in_use: bool,
}


impl Entity {
    fn new() -> Self {
        Self {
            generation: 0,
            is_in_use: false,
        }
    }

    
    fn kill_generation(&mut self) {
        self.generation = self.generation.wrapping_add(1);
        self.is_in_use = false;
    }
}


#[derive(Debug, Clone, Copy)]
pub struct EntityId {
    index: usize,
    generation: usize,
}


#[derive(Debug, Clone, Copy, PartialEq, PartialOrd)]
pub struct ComponentIndex(usize);


#[derive(Debug, Clone)]
struct Component {
    is_init: bool,
    data: Data,
}

impl Component {
    fn empty() -> Self {
        Self {
            is_init: false,
            data: Data::new_unit(),
        }
    }
}


#[derive(Debug)]
pub struct System {
    components: Vec<ComponentIndex>,
    code: u8,
}

impl System {
    pub fn new(components: Vec<ComponentIndex>, code: u8) -> Self { Self { components, code } }
}
