use std::{sync::atomic::{AtomicU32, AtomicU64}, thread, time::Instant};

//use rayon::prelude::{IntoParallelRefMutIterator, IndexedParallelIterator, ParallelIterator};

use rayon::iter::{IntoParallelRefMutIterator, ParallelBridge, ParallelIterator};

use crate::{VM, Object, obj_map::{ObjectMap, ObjectData, ObjectIndex}};

impl VM<'_> {
    pub fn run_garbage_collection(&mut self) {
        let instant = Instant::now();
        
        self.mark();
        self.sweep();

        let elapsed = instant.elapsed();
        println!("took {elapsed:?} to run gc")
        
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
        let free = AtomicU32::new(self.objs.free.0);
        self.objs.raw_mut()
            .iter_mut()
            .enumerate()
            .filter(|(_, object)| !matches!(object.data, ObjectData::Free { .. }))
            .filter(|(_, object)| !object.liveliness.replace(false))
            .filter(|(_, object)| !object.leaked)
            .for_each(|(index, object)| object.data = ObjectData::Free(ObjectIndex::new(free.swap(index as u32, std::sync::atomic::Ordering::Relaxed))));

        self.objs.free = ObjectIndex::new(free.into_inner());
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

            
            | ObjectData::String(_)
            | ObjectData::Free { .. } => (),
        }
    }
}
