use cgmath::Point2;

use crate::storage_traits::{ Battlefield, Ship };

use std::cell::Cell;

#[derive(Clone, Copy)]
pub struct SquareMapNode {
    next : Option<usize>,
    my_square : usize,
}

impl SquareMapNode {
    pub fn new() -> Self {
        SquareMapNode {
            next : None,
            my_square : 0,
        }
    }
}

#[derive(Clone, Copy)]
struct Square {
    start : Option<usize>,
}

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

    fn update_ship_square(&mut self, ship_id : usize, ship : &mut Ship) {
        let sq_id = Self::get_square(ship.core.pos);
        let square = self.squares.get_mut(sq_id).expect("Square map out of range");

        ship.square_map_node.my_square = sq_id;
        match square.start.as_mut() {
            Some(start) => {
                ship.square_map_node.next = Some(*start);
                *start = ship_id
            },
            None => ship.square_map_node.next = None,
        }
    }

    /// Updates the square map data, making all the data up to date.
    pub fn update<'a>(&'a mut self, battlefield : &'a mut Battlefield) -> RelevantSquareMap<'a> {
        battlefield.iter_mut_indices()
        .for_each(|(idx, ship)| self.update_ship_square(idx, ship));

        RelevantSquareMap { me : self, battlefield }
    }
}

pub struct SquareIter<'a, 'b : 'a> {
    square_map : &'a RelevantSquareMap<'b>,
    curr : Option<usize>,
}

impl<'a, 'b : 'a> Iterator for SquareIter<'a, 'b> {
    type Item = (usize, &'a Ship);

    fn next(&mut self) -> Option<Self::Item> {
        match self.curr {
            Some(id) => {
                let ship = self.square_map.battlefield().get(id).unwrap();
                self.curr = ship.square_map_node.next;
                Some((id, ship))
            },
            None => None,
        }
    }
}

pub struct RelevantSquareMap<'a> {
    me : &'a mut SquareMap,
    battlefield : &'a mut Battlefield,
}

impl<'a> RelevantSquareMap<'a> {
    pub fn battlefield(&self) -> &Battlefield { &self.battlefield }

    pub fn get_ship_square(ship : &Ship) -> usize {
        SquareMap::get_square(ship.core.pos)
    }

    pub fn iter_ships_in_square_with_indices<'b>(&'b self, sq_id : usize) -> SquareIter<'b, 'a> 
    where
        'a : 'b
    {
        let curr = self.me.squares[sq_id].start;
        SquareIter {
            square_map : self,
            curr,
        }
    }
}
