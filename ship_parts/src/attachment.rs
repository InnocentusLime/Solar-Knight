use std::collections::HashMap;

use crate::storage_traits::Battlefield;

pub struct AttachmentInfo {
    pub parent_id : usize,
    pub parent_uid : u128,
    pub my_uid : u128,
    // offset : Vector2<f32>,
}

pub struct AttachmentSystem {
    subscribers : HashMap<usize, AttachmentInfo>
}

impl AttachmentSystem {
    pub fn new() -> Self {
        AttachmentSystem {
            subscribers : HashMap::new(),
        }
    }

    // TODO could benefit from `drain_filter`
    pub fn update(&mut self, battlefield : &mut Battlefield) {
        self.subscribers.retain(
            |id, attach| { 
                // TODO panics in debug instead?
                match (battlefield.get(*id), battlefield.get(attach.parent_id)) {
                    (Some(ship), Some(parent)) => (ship.core.uid() == attach.my_uid) && (parent.core.uid() == attach.parent_uid),
                    (_, _) => false,
                }
            }
        );

        self.subscribers.iter()
        .for_each(
            |(id, attach)| {
                let parent_pos = {
                    let parent = battlefield.get(attach.parent_id).unwrap();
                    parent.core.pos
                };
                let ship = battlefield.get_mut(*id).unwrap();
                ship.core.pos = parent_pos; // + offest.aware_of(parent_direction)
            }
        )
    }

    pub fn add_attachment(&mut self, id : usize, info : AttachmentInfo) {
        self.subscribers.insert(id, info);
    }
}
