use std::f32::consts::PI;

use crate::{Angle, Block, Rail, World, get_last_rail_world_position};

pub fn preset_1(world: &mut World, start_angle: Angle) -> Angle {
    world.rails.push(Block::Rail(Rail::new_straight(
        get_last_rail_world_position(world),
        30.0,
        8,
        start_angle,
        false,
    )));

    let last_angle = PI / 8.0;
    world.rails.push(Block::Rail(Rail::new_curved(
        get_last_rail_world_position(world),
        30.0,
        8,
        last_angle,
        PI / 8.0,
        false,
    )));

    last_angle
}

pub fn preset_2(world: &mut World, start_angle: Angle) -> Angle {
    world.rails.push(Block::Rail(Rail::new_straight(
        get_last_rail_world_position(world),
        30.0,
        8,
        start_angle,
        false,
    )));

    let last_angle = PI / 2.0;
    world.rails.push(Block::Rail(Rail::new_curved(
        get_last_rail_world_position(world),
        30.0,
        8,
        last_angle,
        PI / 8.0,
        false,
    )));
    last_angle
}
