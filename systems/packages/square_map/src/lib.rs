use float_ord::FloatOrd;
use cgmath::{ MetricSpace, Point2, vec2 };

use ship_transform::Transform;
use systems_core::{ get_component, get_component_mut, ComponentAccess, Storage, StorageAccessError, DeletionObserver, SpawningObserver, MutationObserver };

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
    fn insert_into_square<Host>(square_id : usize, square : &mut Square, host : &mut Host, idx : usize) -> Result<(), StorageAccessError> 
    where
        Host : Storage,
        Host::Object : ComponentAccess<SquareMapNode>,
    {
        let next_id = square.start;
        square.start = Some(idx);

        let the_inserted = host.get_mut(idx).ok_or(StorageAccessError)?;
        let () = {
            let node = get_component_mut::<SquareMapNode, _>(the_inserted);
            node.prev = None;
            node.next = next_id;
            node.square_id = square_id;
        };

        if let Some(next_id) = next_id {
            get_component_mut::<SquareMapNode, _>(
                host.get_mut(next_id)
                .ok_or(StorageAccessError)?
            ).prev = Some(idx);
        }

        Ok(())
    }

    fn insert<Host>(&mut self, host : &mut Host, idx : usize) -> Result<(), StorageAccessError> 
    where
        Host : Storage,
        Host::Object : ComponentAccess<SquareMapNode> + ComponentAccess<Transform>,
    {
        let pos = get_component::<Transform, _>(host.get(idx).ok_or(StorageAccessError)?).pos;

        // TODO error code
        let square_id = Self::get_square(pos).unwrap();
        // TODO error code (?) this shouldn't be failiing now.
        let square = self.squares.get_mut(square_id).expect("Square ID out of range");

        Self::insert_into_square(square_id, square, host, idx)
    } 

    fn delete<Host>(&mut self, host : &mut Host, idx : usize) -> Result<(), StorageAccessError>
    where
        Host : Storage,
        Host::Object : ComponentAccess<SquareMapNode>,
    {
        let (prev, next, square) = {
            let node = get_component::<SquareMapNode, _>(host.get(idx).ok_or(StorageAccessError)?);
            (node.prev, node.next, node.square_id)
        };

        match prev {
            Some(prev) => 
                get_component_mut::<SquareMapNode, _>(
                    host.get_mut(prev)
                    .ok_or(StorageAccessError)?
                ).next = next
            ,
            None => {
                let square = self.squares.get_mut(square).unwrap();
                square.start = next;
            },
        }

        if let Some(next) = next {
            get_component_mut::<SquareMapNode, _>(
                host.get_mut(next)
                .ok_or(StorageAccessError)?
            ).prev = prev;
        }

        Ok(())
    }

    fn update_data<Host>(&mut self, host : &mut Host, idx : usize) -> Result<(), StorageAccessError> 
    where
        Host : Storage,
        Host::Object : ComponentAccess<SquareMapNode> + ComponentAccess<Transform>,
    {
        let (new_square, current_square) = {
            let obj = host.get(idx).ok_or(StorageAccessError)?;
            // TODO error code
            (
                Self::get_square(get_component::<Transform, _>(obj).pos).unwrap(), 
                get_component::<SquareMapNode, _>(obj).square_id
            )
        };

        if new_square != current_square {
            self.delete(host, idx)?;
            let square = self.squares.get_mut(new_square).expect("Square ID out of range");
            Self::insert_into_square(new_square, square, host, idx)?;
        }

        Ok(())
    }

    fn iter_square_ref<'a, Host>(host : &'a Host, square : &Square) -> impl Iterator<Item = (usize, &'a Host::Object)> + 'a 
    where
        Host : Storage,
        Host::Object : ComponentAccess<SquareMapNode>,
    {
        use std::iter;

        let mut current = square.start;
        iter::from_fn(move || {
            match current {
                Some(id) => {
                    let ship = host.get(id).unwrap();
                    current = get_component::<SquareMapNode, _>(ship).next;
                    Some((id, ship))
                }
                None => None,
            }
        })
    }

    pub fn iter_square<'a, Host>(&'a self, host : &'a Host, id : usize) -> impl Iterator<Item = (usize, &'a Host::Object)> + 'a 
    where
        Host : Storage,
        Host::Object : ComponentAccess<SquareMapNode>,
    {
        let square = self.squares.get(id).unwrap();
        Self::iter_square_ref(host, square)
    }


    // TODO test
    pub fn find_closest<Host, Filter : Fn(&Host::Object)->bool>(
        &self, 
        host : &Host, 
        pos : Point2<f32>, 
        range : f32, filter : Filter
    ) -> Option<usize> 
    where
        Host : Storage,
        Host::Object : ComponentAccess<SquareMapNode> + ComponentAccess<Transform>,
    {
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
                    .map(|(id, ship)| (FloatOrd(pos.distance(get_component::<Transform, _>(ship).pos)), id, ship))
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

impl<Host> MutationObserver<Host> for SquareMap 
where
    Host : Storage,
    Host::Object : ComponentAccess<SquareMapNode> + ComponentAccess<Transform>,
{
    fn on_mutation(&mut self, storage : &mut Host, idx : usize) {
        self.update_data(storage, idx).unwrap()
    }
}

impl<Host> SpawningObserver<Host> for SquareMap 
where
    Host : Storage,
    Host::Object : ComponentAccess<SquareMapNode> + ComponentAccess<Transform>,
{
    fn on_spawn(&mut self, storage : &mut Host, idx : usize) {
        self.insert(storage, idx).unwrap()
    }
}

impl<Host> DeletionObserver<Host> for SquareMap 
where
    Host : Storage,
    Host::Object : ComponentAccess<SquareMapNode> + ComponentAccess<Transform>,
{
    fn on_delete(&mut self, storage : &mut Host, idx : usize) {
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
