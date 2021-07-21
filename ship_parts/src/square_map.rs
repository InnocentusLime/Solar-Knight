use cgmath::Point2;

use crate::storage::{ Ship, MutableStorage };
use crate::storage_traits::*;

#[derive(Clone, Copy)]
pub struct SquareMapNode {
    next : Option<usize>,
    prev : Option<usize>,
    square_id : usize,
}

impl SquareMapNode {
    pub fn new() -> Self {
        SquareMapNode {
            next : None,
            prev : None,
            square_id : 0,
        }
    }
}

#[derive(Clone, Copy, Debug)]
pub struct StorageAccessError;

pub trait SquareMapObject {
    fn pos(&self) -> Point2<f32>;
    fn square_map_node(&self) -> &SquareMapNode;
    fn square_map_node_mut(&mut self) -> &mut SquareMapNode;
}

pub trait SquareMapHost {
    type Object : SquareMapObject;

    fn get(&self, idx : usize) -> Result<&Self::Object, StorageAccessError>;
    fn get_mut(&mut self, idx : usize) -> Result<&mut Self::Object, StorageAccessError>;
}

#[derive(Clone, Copy)]
struct Square {
    start : Option<usize>,
}

// NOTE if it turns out that the current impl is too slow,
// try using a more lazy computation-like appraoch for
// updating the square map.
/// A square map is a special data structure which helps
/// the game narrow down the search for objects near some
/// point. In Solar Knight this helps us optimize things like
/// 1. Collision checking
/// 2. Checking if the user clicked on a ship in debug mode
/// 3. Searching for the closest ship
pub struct SquareMap {
    squares : Vec<Square>,
}

impl SquareMap {
    // Size of the square in units
    const SQUARE_SIDE : f32 = 8.0f32;
    const SQUARE_MAP_SIDE_COUNT : usize = 100usize;

    pub fn new() -> Self {
        SquareMap {
            squares : vec![Square { start : None }; 4 * Self::SQUARE_MAP_SIDE_COUNT * Self::SQUARE_MAP_SIDE_COUNT],
        }
    }

    #[inline]
    pub fn get_square(pos : Point2<f32>) -> usize {
        let sq_x = (pos.x + ((Self::SQUARE_MAP_SIDE_COUNT / 2usize) as f32) * Self::SQUARE_SIDE).div_euclid(Self::SQUARE_SIDE) as usize;
        let sq_y = (pos.y + ((Self::SQUARE_MAP_SIDE_COUNT / 2usize) as f32) * Self::SQUARE_SIDE).div_euclid(Self::SQUARE_SIDE) as usize;

        sq_x + sq_y*Self::SQUARE_MAP_SIDE_COUNT
    }

    #[inline]
    fn insert_into_square<Host : SquareMapHost>(square_id : usize, square : &mut Square, host : &mut Host, idx : usize) -> Result<(), StorageAccessError> {
        let next_id = square.start.map(|x| { square.start = Some(idx); x });

        let the_inserted = host.get_mut(idx)?;
        the_inserted.square_map_node_mut().prev = None;
        the_inserted.square_map_node_mut().next = next_id;
        the_inserted.square_map_node_mut().square_id = square_id;

        if let Some(next_id) = next_id {
            host.get_mut(next_id)?.square_map_node_mut().prev = Some(idx);
        }

        Ok(())
    }

    pub fn insert<Host : SquareMapHost>(&mut self, host : &mut Host, idx : usize) -> Result<(), StorageAccessError> {
        let pos = host.get(idx)?.pos();

        let square_id = Self::get_square(pos);
        // TODO error code
        let square = self.squares.get_mut(square_id).expect("Square ID out of range");

        Self::insert_into_square(square_id, square, host, idx)
    } 

    pub fn delete<Host : SquareMapHost>(&mut self, host : &mut Host, idx : usize) -> Result<(), StorageAccessError> {
        let (prev, next) = {
            let node = host.get(idx)?.square_map_node();
            (node.prev, node.next)
        };

        if let Some(prev) = prev {
            host.get_mut(prev)?.square_map_node_mut().next = next;
        }

        if let Some(next) = next {
            host.get_mut(next)?.square_map_node_mut().prev = prev;
        }

        Ok(())
    }

    pub fn update_data<Host : SquareMapHost>(&mut self, host : &mut Host, idx : usize) -> Result<(), StorageAccessError> {
        let (new_square, current_square) = {
            let obj = host.get(idx)?;
            (Self::get_square(obj.pos()), obj.square_map_node().square_id)
        };

        if new_square != current_square {
            self.delete(host, idx)?;
            let square = self.squares.get_mut(new_square).expect("Square ID out of range");
            Self::insert_into_square(new_square, square, host, idx)?;
        }

        Ok(())
    }
}

impl SquareMapObject for Ship {
    fn pos(&self) -> cgmath::Point2<f32> { self.core.pos }
    fn square_map_node(&self) -> &crate::square_map::SquareMapNode { &self.square_map_node }
    fn square_map_node_mut(&mut self) -> &mut crate::square_map::SquareMapNode { &mut self.square_map_node }
}

impl<'a> SquareMapHost for MutableStorage<'a> {
    type Object = Ship;

    fn get(&self, idx : usize) -> Result<&Self::Object, crate::square_map::StorageAccessError> { 
        self.get(idx).ok_or(crate::square_map::StorageAccessError)
    }
    fn get_mut(&mut self, idx : usize) -> Result<&mut Self::Object, crate::square_map::StorageAccessError> {
        self.get_mut(idx).ok_or(crate::square_map::StorageAccessError)
    }
}

impl MutationObserver for SquareMap {
    fn on_mutation(&mut self, storage : &mut MutableStorage, idx : usize) {
        self.update_data(storage, idx).unwrap()
    }
}

impl SpawningObserver for SquareMap {
    fn on_spawn(&mut self, storage : &mut MutableStorage, idx : usize) {
        self.insert(storage, idx).unwrap()
    }
}

impl DeletionObserver for SquareMap {
    fn on_delete(&mut self, storage : &mut MutableStorage, idx : usize) {
        self.delete(storage, idx).unwrap()
    }
}
