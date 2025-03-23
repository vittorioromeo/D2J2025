use std::f32::consts::PI;

use rand::seq::IndexedRandom;

use crate::{Angle, Block, Fork, ForkSelection, Rail, World, get_last_rail_world_position};

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

fn preset_3(world: &mut World, start_angle: Angle) {
    world.rails.push(Block::Fork(Fork {
        which: ForkSelection::Rail2,
        rail1: Rail::new_straight(
            get_last_rail_world_position(world),
            30.0,
            8,
            // this was `PI + PI / 6.0` before
            start_angle,
            false,
        ),
        rail2: Rail::new_straight(
            get_last_rail_world_position(world),
            30.0,
            8,
            PI + -PI / 6.0,
            true,
        ),
    }));

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
    const MAX: i32 = 3;
    let random_number = random_number(MAX);
    println!("Random number: {}", random_number);

    // We could generalize this but it's not easy
    match random_number {
        1 => preset_1(world, start_angle),
        2 => preset_2(world, start_angle),
        MAX => preset_3(world, start_angle),
        _ => unreachable!(),
    }

    world.rails.last().unwrap().last_angle()
}

fn random_number(max: i32) -> i32 {
    // using rand because macroquad's rand always return the same numbers
    let mut rng = rand::rng();
    let nums: Vec<i32> = (1..=max).collect();

    *nums.choose(&mut rng).unwrap()
}
