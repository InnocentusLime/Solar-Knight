use std::collections::{ HashMap, HashSet };

use ship_transform::Transform;
use systems_core::{ DeletionObserver, MutationObserver, Observation, ComponentAccess, Storage, get_component, get_component_mut };

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

    pub fn update<Host, Observer>(
        &mut self, 
        storage : &mut Observation<Observer, Host>,
    ) 
    where
        Host : Storage,
        Observer : MutationObserver<Host>,
        Host::Object : ComponentAccess<Transform>,
    {
        self.subscribers.iter()
        .for_each(
            |(id, attach)| {
                let parent_pos = {
                    let parent = storage.get(attach.parent_id).unwrap();
                    get_component::<Transform, _>(parent).pos
                };
                storage.mutate(*id,
                |obj, _| {
                    get_component_mut::<Transform, _>(obj).pos = parent_pos; // + offest.aware_of(parent_direction)
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

impl<Host : Storage> DeletionObserver<Host> for AttachmentSystem {
    fn on_delete(&mut self, _storage : &mut Host, idx : usize) {
        // TODO error codes
        let parent_infos = &mut self.parent_infos;
        let subscribers = &mut self.subscribers;

        subscribers.remove(&idx)
        .map_or((), |info| { parent_infos.get_mut(&info.parent_id).unwrap().children.remove(&idx); });

        parent_infos.remove(&idx)
        .map_or((), |info| info.children.into_iter().for_each(|child| { subscribers.remove(&child); }));
    }
}
