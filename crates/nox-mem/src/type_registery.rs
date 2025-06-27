use std::collections::HashMap;

use core::any::TypeId;

pub struct TypeInfo {
    pub type_id: TypeId,
    pub drop_fn: fn(*mut u8),
    pub move_fn: fn(src: *mut u8, dst: *mut u8),
}

fn drop_impl<T>(ptr: *mut u8) {
    unsafe {
        ptr.cast::<T>().drop_in_place();
    }
}

fn move_impl<T>(src: *mut u8, dst: *mut u8) {
    unsafe {
        dst.cast::<T>().write(
            src.cast::<T>().read()
        );
    }
}

pub struct TypeRegistery {
    infos: Vec<TypeInfo>,
    lookup: HashMap<TypeId, u32>,
}

impl TypeRegistery {

    pub fn register<T: 'static>(&mut self) -> u32 {
        let id = TypeId::of::<T>();
        if let Some(idx) = self.lookup.get(&id) {
            *idx
        }
        else {
            let idx = self.lookup.insert(id, self.infos.len() as u32).unwrap();
            self.infos.push(
                TypeInfo {
                    type_id: id,
                    drop_fn: drop_impl::<T>,
                    move_fn: move_impl::<T>,
                }
            );
            idx
        }
    }

    pub fn get_type_info(&self, type_index: u32) -> Option<&TypeInfo> {
        if type_index >= self.infos.len() as u32 {
            None
        }
        else {
            Some(&self.infos[type_index as usize])
        }
    }
}
