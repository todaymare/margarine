use std::{sync::atomic::AtomicU32, time::Instant};

use crate::{VM, Object, obj_map::{ObjectMap, ObjectData, ObjectIndex}};

impl VM<'_> {
    pub fn run_garbage_collection(&mut self) {
        //println!("running gc");
        let instant = Instant::now();
        
        self.mark();
        self.sweep();

        let elapsed = instant.elapsed();
        //println!("took {elapsed:?} to run gc")
    }


    fn mark(&mut self) {
        for object in 0..self.stack.curr {
            let val = self.stack.values[object];
            if val.is_obj() {
                self.objs.get(unsafe { val.as_obj() }).mark(true, &self.objs);
            }
        }
    }


    fn sweep(&mut self) {
        let mut free = self.objs.free;
        self.objs.raw_mut()
            .iter_mut()
            .enumerate()
            .filter(|(_, object)| !matches!(object.data, ObjectData::Free { .. }))
            .filter(|(_, object)| !object.liveliness.replace(false))
            .filter(|(_, object)| !object.leaked)
            .for_each(|(index, object)| {
                object.data = ObjectData::Free(free);
                free = ObjectIndex::new(index as u32);
            });

        self.objs.free = free;
    }
}


impl Object {
    fn mark(&self, mark_as: bool, objects: &ObjectMap) {
        if self.liveliness.replace(mark_as) {
            return
        }

        match &self.data {
              ObjectData::List(fields)
            | ObjectData::FuncRef { captures: fields, .. }
            | ObjectData::Struct { fields } => {
                fields.iter()
                    .filter(|x| x.is_obj())
                    .for_each(|x| objects.get(unsafe { x.as_obj() }).mark(mark_as, objects))
            },

            ObjectData::Dict(hm) => {
                hm.keys()
                    .filter(|x| x.is_obj())
                    .for_each(|x| objects.get(unsafe { x.as_obj() }).mark(mark_as, objects));

                hm.values()
                    .filter(|x| x.is_obj())
                    .for_each(|x| objects.get(unsafe { x.as_obj() }).mark(mark_as, objects));
            }


            ObjectData::Ptr(_) => {}

            
            | ObjectData::String(_)
            | ObjectData::Free { .. } => (),
        }
    }
}
