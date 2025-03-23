use std::f32::consts::PI;

use rand::seq::IndexedRandom;

use crate::{Angle, Block, Rail, World, get_last_rail_world_position};

pub fn preset_1(world: &mut World, start_angle: Angle) {
    world.rails.push(Block::Rail(Rail::new_straight(
        get_last_rail_world_position(world),
        30.0,
        8,
        start_angle,
        false,
    )));

    world.rails.push(Block::Rail(Rail::new_curved(
        get_last_rail_world_position(world),
        30.0,
        8,
        PI / 8.0,
        PI / 8.0,
        false,
    )));

    // IMPROVEMENT: return the last point so we don't need to use get_last_rail_world_position
}

pub fn preset_2(world: &mut World, start_angle: Angle) {
    world.rails.push(Block::Rail(Rail::new_straight(
        get_last_rail_world_position(world),
        30.0,
        8,
        start_angle,
        false,
    )));

    world.rails.push(Block::Rail(Rail::new_curved(
        get_last_rail_world_position(world),
        30.0,
        8,
        PI / 2.0,
        PI / 8.0,
        false,
    )));
}

pub fn preset_random(world: &mut World, start_angle: Angle) -> Angle {
    let random_number = random_number();
    let is_even = random_number % 2 == 0;

    // We could generalize this but it's not easy
    if is_even {
        preset_1(world, start_angle)
    } else {
        preset_2(world, start_angle)
    }

    world.rails.last().unwrap().last_angle()
}

fn random_number() -> i32 {
    // using rand because macroquad's rand always return the same numbers
    let mut rng = rand::rng();
    let nums: Vec<i32> = (1..=2).collect();

    *nums.choose(&mut rng).unwrap()
}
