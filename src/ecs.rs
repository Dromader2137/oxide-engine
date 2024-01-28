use std::{any::Any, cell::{RefMut, RefCell}, borrow::BorrowMut};

pub trait Component {

}

pub trait ComponentVec {
    fn as_any(&self) -> &dyn Any;
    fn as_any_mut(&mut self) -> &mut dyn Any;
    fn push_none(&mut self);
}

pub struct World {
    pub entity_count: usize,
    pub components: Vec<Box<dyn ComponentVec>>,
}

impl World {
    pub fn new() -> World {
        World { entity_count: 0, components: Vec::new() }
    }

    pub fn new_entity(&mut self) -> usize {
        let entity_id = self.entity_count;
        for component_vec in self.components.iter_mut() {
            component_vec.push_none();
        }
        self.entity_count += 1;
        entity_id
    }

    pub fn add_component<Component: 'static>(&mut self, entity_id: usize, component: Component) {
        for component_vec in self.components.iter_mut() {
            if let Some(component_vec) = component_vec
                .as_any_mut()
                .downcast_mut::<RefCell<Vec<Option<Component>>>>()
            {
                component_vec.get_mut()[entity_id] = Some(component);
                return;
            }
        }

        let mut new_component_vec: Vec<Option<Component>> =
            Vec::with_capacity(self.entity_count);

        for _ in 0..self.entity_count {
            new_component_vec.push(None);
        }

        new_component_vec[entity_id] = Some(component);
        self.components
            .push(Box::new(RefCell::new(new_component_vec)));
    }

    pub fn borrow_component_vec_mut<ComponentType: 'static + Clone>(&self) -> Option<RefMut<Vec<Option<ComponentType>>>> {
        for component_vec in self.components.iter() {
            if let Some(component) = component_vec
                .as_any()
                .downcast_ref::<RefCell<Vec<Option<ComponentType>>>>()
            {
                return Some(component.borrow_mut());
            }
        }
        None
    }
}

impl<T: 'static> ComponentVec for RefCell<Vec<Option<T>>> {
    fn as_any(&self) -> &dyn std::any::Any {
        self as &dyn std::any::Any
    }

    fn as_any_mut(&mut self) -> &mut dyn std::any::Any {
        self as &mut dyn std::any::Any
    }

    fn push_none(&mut self) {
        self.get_mut().push(None);
    }
}
