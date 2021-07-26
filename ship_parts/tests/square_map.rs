// Integration test (square map and ship storage)

use ship_parts;
use ship_parts::storage::Storage;
use ship_parts::square_map::SquareMap;
use ship_parts::attachment::AttachmentSystem;
use ship_parts::gun::BulletSystem;

use cgmath::{ vec2, point2 };

#[test]
fn insertion_test() {
    for y in 0..(SquareMap::SQUARE_MAP_SIDE_COUNT) {
        for x in 0..(SquareMap::SQUARE_MAP_SIDE_COUNT) {
            let mut square_map = SquareMap::new();
            let (x, y) = 
                (
                    (x as f32) * SquareMap::SQUARE_SIDE + SquareMap::SQUARE_SIDE / 2.0f32 
                    - (SquareMap::SQUARE_MAP_SIDE_COUNT_HALF as f32) * SquareMap::SQUARE_SIDE,
                    (y as f32) * SquareMap::SQUARE_SIDE + SquareMap::SQUARE_SIDE / 2.0f32 
                    - (SquareMap::SQUARE_MAP_SIDE_COUNT_HALF as f32) * SquareMap::SQUARE_SIDE
                )
            ;
            let mut storage = Storage::new();
            let mut lock = storage.unlock_spawning(&mut square_map);

            let square = SquareMap::get_square(point2(x, y)).unwrap();
            let ship1 = lock.spawn_template_at(0, point2(x, y));
            let ship2 = lock.spawn_template_at(0, point2(x, y));
            let ship3 = lock.spawn_template_at(0, point2(x, y));

            let mut found1 = false;
            let mut found2 = false;
            let mut found3 = false;
            let mut num_iter = 0;
            square_map.iter_square(&storage, square)
            .for_each(|(id, _)| {
                num_iter += 1;
                if id == ship1 { found1 = true; }
                if id == ship2 { found2 = true; }
                if id == ship3 { found3 = true; }
            });

            assert_eq!(num_iter, 3);
            assert!(found1);
            assert!(found2);
            assert!(found3);
        }
    }
} 

#[test]
fn deletion_test() {
    for y in 0..(SquareMap::SQUARE_MAP_SIDE_COUNT) {
        for x in 0..(SquareMap::SQUARE_MAP_SIDE_COUNT) {
            let mut square_map = SquareMap::new();
            // TODO that's no good
            let mut attach_sys = AttachmentSystem::new();
            let mut bullet_sys = BulletSystem::new();

            let (x, y) = 
                (
                    (x as f32) * SquareMap::SQUARE_SIDE + SquareMap::SQUARE_SIDE / 2.0f32 
                    - (SquareMap::SQUARE_MAP_SIDE_COUNT_HALF as f32) * SquareMap::SQUARE_SIDE,
                    (y as f32) * SquareMap::SQUARE_SIDE + SquareMap::SQUARE_SIDE / 2.0f32 
                    - (SquareMap::SQUARE_MAP_SIDE_COUNT_HALF as f32) * SquareMap::SQUARE_SIDE
                )
            ;
            let mut storage = Storage::new();
            let mut lock = storage.unlock_spawning(&mut square_map);

            let square = SquareMap::get_square(point2(x, y)).unwrap();
            let ship1 = lock.spawn_template_at(0, point2(x, y));
            let ship2 = lock.spawn_template_at(0, point2(x, y));
            let ship3 = lock.spawn_template_at(0, point2(x, y));

            let mut lock = storage.unlock_deletion(&mut square_map, &mut attach_sys, &mut bullet_sys);
            lock.delete(ship2);

            let mut found1 = false;
            let mut found2 = false;
            let mut found3 = false;
            let mut num_iter = 0;
            square_map.iter_square(&storage, square)
            .for_each(|(id, _)| {
                num_iter += 1;
                if id == ship1 { found1 = true; }
                if id == ship2 { found2 = true; }
                if id == ship3 { found3 = true; }
            });

            assert_eq!(num_iter, 2);
            assert!(found1);
            assert!(!found2);
            assert!(found3);
        }
    }
} 

#[test]
fn mutation_test() {
    let mut square_map = SquareMap::new();
    // TODO that's no good
    let attach_sys = AttachmentSystem::new();
    let bullet_sys = BulletSystem::new();

    let (x, y) = (3, 6);
    let (x, y) = 
        (
            (x as f32) * SquareMap::SQUARE_SIDE + SquareMap::SQUARE_SIDE / 2.0f32 
            - (SquareMap::SQUARE_MAP_SIDE_COUNT_HALF as f32) * SquareMap::SQUARE_SIDE,
            (y as f32) * SquareMap::SQUARE_SIDE + SquareMap::SQUARE_SIDE / 2.0f32 
            - (SquareMap::SQUARE_MAP_SIDE_COUNT_HALF as f32) * SquareMap::SQUARE_SIDE
        )
    ;
    let mut storage = Storage::new();
    let mut lock = storage.unlock_spawning(&mut square_map);

    let ship1 = lock.spawn_template_at(0, point2(x, y));
    let ship2 = lock.spawn_template_at(0, point2(x, y));
    let ship3 = lock.spawn_template_at(0, point2(x, y));

    let mut lock = storage.unlock_mutations(&mut square_map);
    lock.mutate(ship2, |ship| ship.core.pos += vec2(SquareMap::SQUARE_SIDE, 0.0f32));

    let mut found1 = false;
    let mut found2 = false;
    let mut found3 = false;
    let mut num_iter = 0;
    
    let square = SquareMap::get_square(point2(x, y)).unwrap();
    num_iter = 0;
    square_map.iter_square(&storage, square)
    .for_each(|(id, _)| {
        num_iter += 1;
        if id == ship1 { found1 = true; }
        if id == ship2 { found2 = true; }
        if id == ship3 { found3 = true; }
    });
    assert_eq!(num_iter, 2);
    assert!(found1);
    assert!(!found2);
    assert!(found3);

    found1 = false;
    found2 = false;
    found3 = false;

    let square = SquareMap::get_square(point2(x + SquareMap::SQUARE_SIDE, y)).unwrap();
    num_iter = 0;
    square_map.iter_square(&storage, square)
    .for_each(|(id, _)| {
        num_iter += 1;
        if id == ship1 { found1 = true; }
        if id == ship2 { found2 = true; }
        if id == ship3 { found3 = true; }
    });
    assert_eq!(num_iter, 1);
    assert!(!found1);
    assert!(found2);
    assert!(!found3);
} 
