use macroquad::prelude::*;
use std::f32::consts::PI;

use crate::{Block, Rail, World, get_last_rail_world_position};

pub fn preset_1(world: &mut World, start_angle: f32) {
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
        0.0,
        PI / 8.0,
        false,
    )));
}
