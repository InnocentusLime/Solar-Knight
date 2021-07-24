use std::collections::{ HashMap, HashSet };

use crate::storage::MutableStorage;
use crate::storage_traits::{ DeletionObserver, MutationObserver, Observation };

#[derive(Clone, Copy)]
pub struct AttachmentInfo {
    pub parent_id : usize,
    // offset : Vector2<f32>,
}

#[derive(Clone)]
struct ParentInfo {
    children : HashSet<usize>,
}

// TODO bench this versus the UID strategy
pub struct AttachmentSystem {
    subscribers : HashMap<usize, AttachmentInfo>,
    parent_infos : HashMap<usize, ParentInfo>,
}

impl AttachmentSystem {
    pub fn new() -> Self {
        AttachmentSystem {
            subscribers : HashMap::new(),
            parent_infos : HashMap::new(),
        }
    }

    pub fn update<Observer : MutationObserver>(
        &mut self, 
        storage : &mut Observation<Observer>,
    ) {
        self.subscribers.iter()
        .for_each(
            |(id, attach)| {
                let parent_pos = {
                    let parent = storage.get(attach.parent_id).unwrap();
                    parent.core.pos
                };
                storage.mutate(*id,
                |ship| {
                    ship.core.pos = parent_pos; // + offest.aware_of(parent_direction)
                });
            }
        )
    }

    pub fn add_attachment(&mut self, id : usize, info : AttachmentInfo) {
        self.subscribers.insert(id, info);
        self.parent_infos.entry(info.parent_id)
        .or_insert(ParentInfo { children : HashSet::new() })
        .children.insert(id);
    }
}

impl DeletionObserver for AttachmentSystem {
    fn on_delete(&mut self, storage : &mut MutableStorage, idx : usize) {
        // TODO error codes
        let parent_infos = &mut self.parent_infos;
        let subscribers = &mut self.subscribers;

        subscribers.remove(&idx)
        .map_or((), |info| { parent_infos.get_mut(&info.parent_id).unwrap().children.remove(&idx); });

        parent_infos.remove(&idx)
        .map_or((), |info| info.children.into_iter().for_each(|child| { subscribers.remove(&child); }));
    }
}
