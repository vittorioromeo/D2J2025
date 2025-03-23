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

    world.rails.push(Block::Rail(Rail::new_curved(
        get_last_rail_world_position(world),
        30.0,
        8,
        PI / 8.0,
        PI / 8.0,
        false,
    )));

    // TODO: take last two points, calculate angle between them and return it. also return the last point so we don't need to use get_last_rail_world_position

    // Calculate angle between last two points
    world.rails.last().unwrap().last_angle()
}

pub fn preset_2(world: &mut World, start_angle: Angle) -> Angle {
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

    world.rails.last().unwrap().last_angle()
}
