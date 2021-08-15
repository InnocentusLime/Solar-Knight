use float_ord::FloatOrd;
use cgmath::{ MetricSpace, Point2 };

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

struct RingIter<'a> {
    done : bool,
    center : usize,
    x : i32,
    y : i32,
    depth : u32,
    square_map : &'a SquareMap,
}

impl<'a> RingIter<'a> {
    fn new(square_map : &'a SquareMap, center : usize, depth : u32) -> Self {
        RingIter {
            square_map,
            done : false,
            center,
            x : depth as i32,
            y : depth as i32,
            depth,
        }
    }
}

impl<'a> Iterator for RingIter<'a> {
    type Item = &'a Square;

    fn next(&mut self) -> Option<Self::Item> {
        //println!("Step start. depth={} x={} y={} done={}", self.depth, self.x, self.y, self.done);
        // Start of the search --- we found no square
        let mut square = None;

        while square.is_none() && !self.done {
            square = 
                SquareMap::get_adj_square(self.center, self.x, self.y)
                .and_then(|id| self.square_map.squares.get(id))
            ;

            // Pick the next square
            if self.y == self.depth as i32 && (-(self.depth as i32)) < self.x { 
                self.x -= 1; 
                continue 
            }
            if self.x == -(self.depth as i32) && (-(self.depth as i32)) < self.y { 
                self.y -= 1; 
                continue 
            }
            if self.y == -(self.depth as i32) && self.x < self.depth as i32 { 
                self.x += 1; 
                continue 
            }
            if self.x == self.depth as i32 && self.y < self.depth as i32 { 
                self.y += 1;
                // TODO redundant conditions?
                self.done = (self.x == self.y) && (self.x >= 0) && (self.y >= 0);
                continue 
            }
            self.done = (self.x == 0) && (self.y == 0);
        }
            
        // As a result, this will be `None`, when depth is exhausted
        // and will be `Some` otherwise
        square
    }
}

struct SquareMapIterBase<'a, Host> 
where
    Host : Storage,
    Host::Object : ComponentAccess<SquareMapNode> + ComponentAccess<Transform>,
{
    top_depth : u32,
    depth : u32,
    square : usize,
    square_map : &'a SquareMap,
    host : &'a Host,
    pos : Point2<f32>,
    range : f32,
}

impl<'a, Host> SquareMapIterBase<'a, Host> 
where
    Host : Storage,
    Host::Object : ComponentAccess<SquareMapNode> + ComponentAccess<Transform>,
{
    fn new(
        pos : Point2<f32>,
        range : f32,
        square : usize, 
        top_depth : u32, 
        square_map : &'a SquareMap, 
        host : &'a Host
    ) -> Self {
        SquareMapIterBase {
            depth : 0,
            pos,
            range,
            square,
            top_depth,
            square_map,
            host,
        }
    }
}

impl<'a, Host> Iterator for SquareMapIterBase<'a, Host>
where
    Host : Storage,
    Host::Object : ComponentAccess<SquareMapNode> + ComponentAccess<Transform>,
{
    type Item = (&'a SquareMap, &'a Host, u32, usize, Point2<f32>, f32);

    fn next(&mut self) -> Option<Self::Item> {
        if self.depth > self.top_depth { return None }

        let result = Some((self.square_map, self.host, self.depth, self.square, self.pos, self.range));
        self.depth += 1;
        result
    }
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

// TODO proper square iterator (reason: it can go backwards)
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
    fn square_xy_to_idx(x : usize, y : usize) -> Option<usize> {
        if 
            x < Self::SQUARE_MAP_SIDE_COUNT &&
            y < Self::SQUARE_MAP_SIDE_COUNT
        {
            Some(
                x +
                y * Self::SQUARE_MAP_SIDE_COUNT
            )
        } else { None }
    }

    #[inline]
    fn square_idx_to_xy(idx : usize) -> Option<(usize, usize)> {
        if idx < Self::SQUARE_MAP_SIDE_COUNT * Self::SQUARE_MAP_SIDE_COUNT {
            Some((idx % Self::SQUARE_MAP_SIDE_COUNT, idx / Self::SQUARE_MAP_SIDE_COUNT))
        } else { None }
    }

    #[inline]
    fn get_adj_square(square_id : usize, dx : i32, dy : i32) -> Option<usize> {
        let (x, y) = Self::square_idx_to_xy(square_id)?;
        let (x, y) = (x as i32 + dx, y as i32 + dy);
        if 
            x < 0 || y < 0 ||
            (Self::SQUARE_MAP_SIDE_COUNT as i32) < x ||
            (Self::SQUARE_MAP_SIDE_COUNT as i32) < y
        {
            None
        } else { 
            Self::square_xy_to_idx(x as usize, y as usize) 
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

    #[inline]
    pub fn iter_zone<'a, Host>(
        &'a self,
        host : &'a Host,
        pos : Point2<f32>,
        range : f32,
    ) -> impl Iterator<Item = (usize, &'a Host::Object)> + 'a
    where
        Host : Storage,
        Host::Object : ComponentAccess<SquareMapNode> + ComponentAccess<Transform>,
    {
        let depth_limit = (range / Self::SQUARE_SIDE).ceil() as u32;
        let point_square = Self::get_square(pos).unwrap();

        // The borrow checker is still a tad dumb. So
        // instead I create a iterator struct to keep
        // all the state.
        SquareMapIterBase::new(
            pos,
            range,
            point_square,
            depth_limit,
            self,
            host
        )
        .map(|(map, host, depth, square, pos, range)| {
            // We have to do a handful of `move`s here
            // because the closures will try to borrow
            // my stuff otherwise
            RingIter::new(map, square, depth)
            .flat_map(move |square| Self::iter_square_ref(host, square))
            .filter(move |(_, obj)| pos.distance(get_component::<Transform, _>(*obj).pos) <= range)
        })
        .flatten()
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
        let depth_limit = (range / Self::SQUARE_SIDE).ceil() as u32;
        let range = FloatOrd(range);
        let point_square = Self::get_square(pos).unwrap();

        // TODO this is a mess. Make it cleaner. (maybe use the struct from `iter_zone`)
        (0..=depth_limit)
        .filter_map(|depth| {
            RingIter::new(self, point_square, depth)
            .flat_map(|square| Self::iter_square_ref(host, square))
            .map(|(id, obj)| (FloatOrd(pos.distance(get_component::<Transform, _>(obj).pos)), id, obj))
            .filter(|x| x.0 <= range && filter(x.2))
            .min_by_key(|x| x.0)
            .map(|x| x.1)
        })
        .next()
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

    fn square_xy_to_sample_point(x : u32, y : u32) -> Point2<f32> {
        let (x, y) = 
            (
                (x as f32) * SquareMap::SQUARE_SIDE + SquareMap::SQUARE_SIDE / 2.0f32 
                - (SquareMap::SQUARE_MAP_SIDE_COUNT_HALF as f32) * SquareMap::SQUARE_SIDE,
                (y as f32) * SquareMap::SQUARE_SIDE + SquareMap::SQUARE_SIDE / 2.0f32 
                - (SquareMap::SQUARE_MAP_SIDE_COUNT_HALF as f32) * SquareMap::SQUARE_SIDE
            )
        ;
        point2(x, y)
    }

    #[test]
    fn get_square_test_x_axis() {
        for y in 0..SquareMap::SQUARE_MAP_SIDE_COUNT {
            for x in 0..(SquareMap::SQUARE_MAP_SIDE_COUNT-1) {
                let sample = square_xy_to_sample_point(x as u32, y as u32);
                let sq1 = SquareMap::get_square(sample).unwrap();
                let sq2 = SquareMap::get_square(point2(sample.x + SquareMap::SQUARE_SIDE, sample.y)).unwrap();
                assert_eq!(sq2, sq1 + 1);
            }
        }
    }
    
    #[test]
    fn get_square_test_y_axis() {
        for y in 0..(SquareMap::SQUARE_MAP_SIDE_COUNT-1) {
            for x in 0..(SquareMap::SQUARE_MAP_SIDE_COUNT) {
                let sample = square_xy_to_sample_point(x as u32, y as u32);
                let sq1 = SquareMap::get_square(sample).unwrap();
                let sq2 = SquareMap::get_square(point2(sample.x, sample.y + SquareMap::SQUARE_SIDE)).unwrap();
                assert_eq!(sq2, sq1 + SquareMap::SQUARE_MAP_SIDE_COUNT);
            }
        }
    }

    #[test]
    fn square_xy_idx_conv() {
        for x in 0..SquareMap::SQUARE_MAP_SIDE_COUNT {
            for y in 0..SquareMap::SQUARE_MAP_SIDE_COUNT {
                let idx = SquareMap::square_xy_to_idx(x, y).unwrap();
                let (nx, ny) = SquareMap::square_idx_to_xy(idx).unwrap();
                assert_eq!(x, nx);
                assert_eq!(y, ny);
            }
        }
    }

    #[test]
    fn sqaure_idx_xy_conv() {
        for idx in 0..(SquareMap::SQUARE_MAP_SIDE_COUNT*SquareMap::SQUARE_MAP_SIDE_COUNT) {
            let (x, y) = SquareMap::square_idx_to_xy(idx).unwrap();
            let nidx = SquareMap::square_xy_to_idx(x, y).unwrap();
            assert_eq!(idx, nidx);
        }
    }

    #[test]
    fn get_adj_square_correct() {
        let x = 45u32;
        let y = 34u32;
        let idx_orig = SquareMap::square_xy_to_idx(x as usize, y as usize).unwrap();

        let p = square_xy_to_sample_point(x, y);
        let p_adj = point2(p.x + SquareMap::SQUARE_SIDE*3.0f32, p.y + SquareMap::SQUARE_SIDE*5.0f32);

        let idx1 = SquareMap::get_square(p_adj).unwrap();
        let idx2 = SquareMap::get_adj_square(idx_orig, 3i32, 5i32).unwrap();

        assert_eq!(idx1, idx2);
    }
}
