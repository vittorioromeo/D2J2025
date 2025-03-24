use std::f32::consts::PI;

use macroquad::math::Vec2;
use rand::seq::IndexedRandom;

use crate::{
    Angle, Block, Fork, ForkSelection, Letter, Rail, World, get_last_rail_world_position,
    get_last_rail_world_start_angle,
};

pub fn get_flipped_mult(flipped: bool) -> f32 {
    if flipped { 1.0 } else { -1.0 }
}

pub fn make_rail_straight(position: Vec2, start_angle: Angle) -> Rail {
    Rail::new_straight(position, start_angle, false, 30.0, 8)
}

pub fn make_rail_90_turn(position: Vec2, start_angle: Angle, is_wall: bool, flipped: bool) -> Rail {
    Rail::new_curved(
        position,
        start_angle,
        is_wall,
        30.0,
        7,
        get_flipped_mult(flipped) * PI / 16.0,
    )
}

pub fn make_rail_u_turn(position: Vec2, start_angle: Angle, is_wall: bool, flipped: bool) -> Rail {
    Rail::new_curved(
        position,
        start_angle,
        is_wall,
        30.0,
        5,
        get_flipped_mult(flipped) * PI / 8.0,
    )
}

pub fn preset_0_straight(world: &mut World) {
    world.rails.push(Block::Rail(make_rail_straight(
        get_last_rail_world_position(world),
        get_last_rail_world_start_angle(world),
    )));
}

pub fn preset_1_u_turn(world: &mut World) {
    world.rails.push(Block::Rail(make_rail_u_turn(
        get_last_rail_world_position(world),
        get_last_rail_world_start_angle(world),
        false,
        random_number(2) == 1,
    )));
}

pub fn preset_2_90_turn(world: &mut World) {
    world.rails.push(Block::Rail(make_rail_90_turn(
        get_last_rail_world_position(world),
        get_last_rail_world_start_angle(world),
        false,
        random_number(2) == 1,
    )));
}

pub fn random_letter() -> Letter {
    match random_number(3) {
        1 => Letter::A,
        2 => Letter::B,
        _ => Letter::C,
    }
}

pub fn preset_3_fork_90_symmetrical_turn(world: &mut World) {
    let position = get_last_rail_world_position(world);
    let start_angle = get_last_rail_world_start_angle(world);

    let random_bool = random_number(2) == 1;
    let random_bool2 = random_number(2) == 1;

    world.rails.push(Block::Fork(Fork {
        which: ForkSelection::Rail2,
        rail1: make_rail_90_turn(position, start_angle, random_bool, random_bool2),
        rail2: make_rail_90_turn(position, start_angle, !random_bool, !random_bool2),
        letter: random_letter(),
    }));
}

pub fn preset_4_fork_u_turn_symmetrical_turn(world: &mut World) {
    let position = get_last_rail_world_position(world);
    let start_angle = get_last_rail_world_start_angle(world);

    let random_bool = random_number(2) == 1;
    let random_bool2 = random_number(2) == 1;

    world.rails.push(Block::Fork(Fork {
        which: ForkSelection::Rail2,
        rail1: make_rail_u_turn(position, start_angle, random_bool, random_bool2),
        rail2: make_rail_u_turn(position, start_angle, !random_bool, !random_bool2),
        letter: random_letter(),
    }));
}

pub fn preset_random(world: &mut World) {
    if random_number(100) >= 50 {
        preset_0_straight(world);
    } else {
        if random_number(100) >= 75 {
            if random_number(100) >= 75 {
                preset_2_90_turn(world);
                preset_0_straight(world);
            } else {
                preset_1_u_turn(world);
                preset_0_straight(world);
            }
        } else {
            if random_number(100) >= 75 {
                preset_3_fork_90_symmetrical_turn(world);
                preset_0_straight(world);
            } else {
                preset_4_fork_u_turn_symmetrical_turn(world);
                preset_0_straight(world);
            }
        }
    }
}

fn random_number(max: i32) -> i32 {
    // using rand because macroquad's rand always return the same numbers
    let mut rng = rand::rng();
    let nums: Vec<i32> = (1..=max).collect();

    *nums.choose(&mut rng).unwrap()
}
