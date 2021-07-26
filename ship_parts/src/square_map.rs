use float_ord::FloatOrd;
use cgmath::{ MetricSpace, Point2, vec2 };

use crate::storage_traits::*;
use crate::storage::{ Ship, Storage, MutableStorage };

#[derive(Clone, Copy, Debug)]
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

    #[inline]
    pub fn square_id(&self) -> usize { self.square_id }
}

#[derive(Clone, Copy, Debug)]
pub struct StorageAccessError;

pub trait SquareMapObject : core::fmt::Debug {
    fn pos(&self) -> Point2<f32>;
    fn square_map_node(&self) -> &SquareMapNode;
    fn square_map_node_mut(&mut self) -> &mut SquareMapNode;
}

pub trait SquareMapHost {
    type Object : SquareMapObject;

    fn get(&self, idx : usize) -> Result<&Self::Object, StorageAccessError>;
}

pub trait SquareMapHostMut : SquareMapHost {
    fn get_mut(&mut self, idx : usize) -> Result<&mut Self::Object, StorageAccessError>;
}

#[derive(Clone, Copy, Debug)]
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
    pub const SQUARE_SIDE : f32 = 1.0f32;
    pub const SQUARE_MAP_SIDE_COUNT_HALF : usize = 75usize;
    pub const SQUARE_MAP_SIDE_COUNT : usize = Self::SQUARE_MAP_SIDE_COUNT_HALF * 2usize;

    pub fn new() -> Self {
        SquareMap {
            squares : vec![Square { start : None }; Self::SQUARE_MAP_SIDE_COUNT * Self::SQUARE_MAP_SIDE_COUNT],
        }
    }

    #[inline]
    pub fn get_square(pos : Point2<f32>) -> Option<usize> {
        let sq_x = (pos.x + (Self::SQUARE_MAP_SIDE_COUNT_HALF as f32) * Self::SQUARE_SIDE).div_euclid(Self::SQUARE_SIDE);
        let sq_y = (pos.y + (Self::SQUARE_MAP_SIDE_COUNT_HALF as f32) * Self::SQUARE_SIDE).div_euclid(Self::SQUARE_SIDE);

        if 
            sq_x < 0.0f32 || sq_x < 0.0f32 || 
            Self::SQUARE_SIDE * (Self::SQUARE_MAP_SIDE_COUNT as f32) < sq_x || 
            Self::SQUARE_SIDE * (Self::SQUARE_MAP_SIDE_COUNT as f32) < sq_y 
        { 
            None
        } else { Some(sq_x as usize + sq_y as usize*Self::SQUARE_MAP_SIDE_COUNT) }
    }

    #[inline]
    fn insert_into_square<Host : SquareMapHostMut>(square_id : usize, square : &mut Square, host : &mut Host, idx : usize) -> Result<(), StorageAccessError> {
        let next_id = square.start;
        square.start = Some(idx);

        let the_inserted = host.get_mut(idx)?;
        the_inserted.square_map_node_mut().prev = None;
        the_inserted.square_map_node_mut().next = next_id;
        the_inserted.square_map_node_mut().square_id = square_id;

        if let Some(next_id) = next_id {
            host.get_mut(next_id)?.square_map_node_mut().prev = Some(idx);
        }

        Ok(())
    }

    pub fn insert<Host : SquareMapHostMut>(&mut self, host : &mut Host, idx : usize) -> Result<(), StorageAccessError> {
        let pos = host.get(idx)?.pos();

        // TODO error code
        let square_id = Self::get_square(pos).unwrap();
        // TODO error code (?) this shouldn't be failiing now.
        let square = self.squares.get_mut(square_id).expect("Square ID out of range");

        Self::insert_into_square(square_id, square, host, idx)
    } 

    pub fn delete<Host : SquareMapHostMut>(&mut self, host : &mut Host, idx : usize) -> Result<(), StorageAccessError> {
        let (prev, next, square) = {
            let node = host.get(idx)?.square_map_node();
            (node.prev, node.next, node.square_id)
        };

        match prev {
            Some(prev) => host.get_mut(prev)?.square_map_node_mut().next = next,
            None => {
                let square = self.squares.get_mut(square).unwrap();
                square.start = next;
            },
        }

        if let Some(next) = next {
            host.get_mut(next)?.square_map_node_mut().prev = prev;
        }

        Ok(())
    }

    pub fn update_data<Host : SquareMapHostMut>(&mut self, host : &mut Host, idx : usize) -> Result<(), StorageAccessError> {
        let (new_square, current_square) = {
            let obj = host.get(idx)?;
            // TODO error code
            (Self::get_square(obj.pos()).unwrap(), obj.square_map_node().square_id)
        };

        if new_square != current_square {
            self.delete(host, idx)?;
            let square = self.squares.get_mut(new_square).expect("Square ID out of range");
            Self::insert_into_square(new_square, square, host, idx)?;
        }

        Ok(())
    }

    fn iter_square_ref<'a, Host : SquareMapHost>(host : &'a Host, square : &Square) -> impl Iterator<Item = (usize, &'a Host::Object)> + 'a {
        use std::iter;

        let mut current = square.start;
        iter::from_fn(move || {
            match current {
                Some(id) => {
                    let ship = host.get(id).unwrap();
                    current = ship.square_map_node().next;
                    Some((id, ship))
                }
                None => None,
            }
        })
    }

    pub fn iter_square<'a, Host : SquareMapHost>(&'a self, host : &'a Host, id : usize) -> impl Iterator<Item = (usize, &'a Host::Object)> + 'a {
        let square = self.squares.get(id).unwrap();
        Self::iter_square_ref(host, square)
    }


    // TODO test
    pub fn find_closest<Host : SquareMapHost, Filter : Fn(&Host::Object)->bool>(
        &self, 
        host : &Host, 
        pos : Point2<f32>, 
        range : f32, filter : Filter
    ) -> Option<usize> {
        let depth_limit = (range / Self::SQUARE_SIDE).ceil() as usize;
        let range = FloatOrd(range);
        
        let mut depth = 0;
        let (mut x, mut y) : (i32, i32) = (0, 0);
        let mut result = None;
        loop {
            // 0. If the depth is too far --- break
            if depth > depth_limit { break; } 

            // 1. Compute the result
            // TODO optimize with a proc to get an adjecent square which uses **no** floats
            if let Some(square) = Self::get_square(pos + Self::SQUARE_SIDE*vec2(x as f32, y as f32)) {
                self.squares.get(square)
                .map(|square| {
                    Self::iter_square_ref(host, square)
                    .map(|(id, ship)| (FloatOrd(pos.distance(ship.pos())), id, ship))
                    .filter(|x| x.0 <= range && filter(x.2))
                    .min_by_key(|x| x.0)
                    .map(|(dist, id, _)| {
                        match &mut result {
                            Some(x) => *x = std::cmp::min_by_key(*x, (dist, id), |y| y.0),
                            None => result = Some((dist, id)),
                        }
                    })
                });
            }

            // 2. If the next iteration will enter the new depth and the result was found --- break
            if x == y && result.is_some() { break; }

            // 3. Next step
            if x == y && 0 <= x && 0 <= y { depth += 1; y += 1; continue }
            if y == depth as i32 && (-(depth as i32)) < x { x -= 1; continue }
            if x == -(depth as i32) && (-(depth as i32)) < y { y -= 1; continue }
            if y == -(depth as i32) && x < depth as i32 { x += 1; continue }
            if x == depth as i32 && y < depth as i32 { y += 1; continue }
        }

        result.map(|x| x.1)
    }
}

impl SquareMapObject for Ship {
    fn pos(&self) -> cgmath::Point2<f32> { self.core.pos }
    fn square_map_node(&self) -> &crate::square_map::SquareMapNode { &self.square_map_node }
    fn square_map_node_mut(&mut self) -> &mut crate::square_map::SquareMapNode { &mut self.square_map_node }
}

impl SquareMapHost for Storage {
    type Object = Ship;

    fn get(&self, idx : usize) -> Result<&Self::Object, crate::square_map::StorageAccessError> { 
        self.get(idx).ok_or(crate::square_map::StorageAccessError)
    }
}

impl<'a> SquareMapHost for MutableStorage<'a> {
    type Object = Ship;

    fn get(&self, idx : usize) -> Result<&Self::Object, crate::square_map::StorageAccessError> { 
        self.get(idx).ok_or(crate::square_map::StorageAccessError)
    }
}

impl<'a> SquareMapHostMut for MutableStorage<'a> {
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

#[cfg(test)]
mod tests {
    use super::*;
    use cgmath::point2;

    #[test]
    fn get_square_test_x_axis() {
        for y in 0..SquareMap::SQUARE_MAP_SIDE_COUNT {
            for x in 0..(SquareMap::SQUARE_MAP_SIDE_COUNT-1) {
                let (x, y) = 
                    (
                        (x as f32) * SquareMap::SQUARE_SIDE + SquareMap::SQUARE_SIDE / 2.0f32 
                        - (SquareMap::SQUARE_MAP_SIDE_COUNT_HALF as f32) * SquareMap::SQUARE_SIDE,
                        (y as f32) * SquareMap::SQUARE_SIDE + SquareMap::SQUARE_SIDE / 2.0f32 
                        - (SquareMap::SQUARE_MAP_SIDE_COUNT_HALF as f32) * SquareMap::SQUARE_SIDE
                    )
                ;
                let sq1 = SquareMap::get_square(point2(x, y)).unwrap();
                let sq2 = SquareMap::get_square(point2(x + SquareMap::SQUARE_SIDE, y)).unwrap();
                assert_eq!(sq2, sq1 + 1);
            }
        }
    }
    
    #[test]
    fn get_square_test_y_axis() {
        for y in 0..(SquareMap::SQUARE_MAP_SIDE_COUNT-1) {
            for x in 0..(SquareMap::SQUARE_MAP_SIDE_COUNT) {
                let (x, y) = 
                    (
                        (x as f32) * SquareMap::SQUARE_SIDE + SquareMap::SQUARE_SIDE / 2.0f32 
                        - (SquareMap::SQUARE_MAP_SIDE_COUNT_HALF as f32) * SquareMap::SQUARE_SIDE,
                        (y as f32) * SquareMap::SQUARE_SIDE + SquareMap::SQUARE_SIDE / 2.0f32 
                        - (SquareMap::SQUARE_MAP_SIDE_COUNT_HALF as f32) * SquareMap::SQUARE_SIDE
                    )
                ;
                let sq1 = SquareMap::get_square(point2(x, y)).unwrap();
                let sq2 = SquareMap::get_square(point2(x, y + SquareMap::SQUARE_SIDE)).unwrap();
                assert_eq!(sq2, sq1 + SquareMap::SQUARE_MAP_SIDE_COUNT);
            }
        }
    }
}
