use std::{cell::Cell, collections::HashMap, fmt::Display, ops::{Index, IndexMut}};

use crate::Reg;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Ord, PartialOrd)]
pub struct ObjectIndex(pub u32);


pub struct ObjectMap {
    pub(crate) objs: Vec<Object>,
    pub(crate) free: ObjectIndex,
}


#[derive(Debug)]
pub struct Object {
    pub(crate) liveliness: Cell<bool>,
    pub(crate) leaked: bool,
    pub(crate) data: ObjectData,

}


#[derive(Debug)]
pub enum ObjectData {
    Struct {
        fields: Vec<Reg>,
    },


    List(Vec<Reg>),


    String(Box<str>),


    Dict(HashMap<Reg, Reg>),


    FuncRef {
        func: u32,
        captures: Vec<Reg>,
    },


    Free(ObjectIndex),
}


impl Object {
    pub fn as_fields(&self) -> &[Reg] {
        match &self.data {
            ObjectData::Struct { fields } => &fields,
            _ => unreachable!(),
        }
    }


    pub fn as_mut_fields(&mut self) -> &mut [Reg] {
        match &mut self.data {
            ObjectData::Struct { fields } => &mut *fields,
            _ => unreachable!(),
        }
    }

    pub fn as_str(&self) -> &str {
        match &self.data {
            ObjectData::String(str) => &str,
            _ => unreachable!(),
        }
    }

    pub fn as_list(&self) -> &[Reg] {
        match &self.data {
            ObjectData::List(vals) => &vals,
            _ => unreachable!(),
        }
    }

    pub fn as_mut_list(&mut self) -> &mut Vec<Reg> {
        match &mut self.data {
            ObjectData::List(vals) => vals,
            _ => unreachable!(),
        }
    }

    pub fn as_hm(&mut self) -> &mut HashMap<Reg, Reg> {
        match &mut self.data {
            ObjectData::Dict(vals) => vals,
            _ => unreachable!(),
        }
    }
}


impl ObjectIndex {
    pub(crate) fn new(index: u32) -> Self { Self(index as _) }
}


impl Display for ObjectIndex {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "[@{}]", self.0)
    }
}



impl ObjectMap {
    pub(crate) fn new(space: usize) -> Self {
        Self {
            free: ObjectIndex::new(0),
            objs: (0..space).map(|x|
                Object::new(ObjectData::Free(
                    ObjectIndex::new(((x + 1) % space) as u32)
                )
            )).collect(),
        }
    }


    /// Inserts an object to the object heap
    ///
    /// # Errors
    /// - If out of memory
    #[inline]
    pub(crate) fn put(&mut self, object: Object) -> Result<ObjectIndex, Object> {
        let index = self.free;
        let v = self.get_mut(self.free);
        let repl = std::mem::replace(v, object);

        match repl.data {
            ObjectData::Free(next) => {
                self.free = next;
                Ok(index)
            },

            _ => {
                let object = std::mem::replace(v, repl);
                Err(object)
            }
        }
    }


    /// Get an object from the object heap
    #[inline(always)]
    pub fn get(&self, index: ObjectIndex) -> &Object {
        &self.objs[index.0 as usize]
    }


    /// Get a mutable object from the object heap
    #[inline(always)]
    pub fn get_mut(&mut self, index: ObjectIndex) -> &mut Object {
        &mut self.objs[index.0 as usize]
    }


    #[inline]
    pub(crate) fn raw(&self) -> &[Object] {
        &self.objs
    }

    
    #[inline]
    pub(crate) fn raw_mut(&mut self) -> &mut [Object] {
        &mut self.objs
    }
}



impl Object {
    pub fn new(data: impl Into<ObjectData>) -> Self { Self { liveliness: Cell::new(false), leaked: false, data: data.into() } }
}


impl Index<ObjectIndex> for ObjectMap {
    type Output = Object;

    fn index(&self, index: ObjectIndex) -> &Self::Output {
        self.get(index)
    }
}


impl IndexMut<ObjectIndex> for ObjectMap {
    fn index_mut(&mut self, index: ObjectIndex) -> &mut Self::Output {
        self.get_mut(index)
    }
}
