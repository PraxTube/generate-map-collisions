use bevy::prelude::*;

fn wrap(a: i32, b: i32) -> usize {
    if a < 0 {
        (a % b + b) as usize
    } else {
        (a % b) as usize
    }
}

pub fn at(poly: &Vec<Vec2>, index: i32) -> Vec2 {
    poly[wrap(index, poly.len() as i32)]
}

pub fn area(a: Vec2, b: Vec2, c: Vec2) -> f32 {
    ((b.x - a.x) * (c.y - a.y)) - ((c.x - a.x) * (b.y - a.y))
}

pub fn left(a: Vec2, b: Vec2, c: Vec2) -> bool {
    area(a, b, c) > 0.0
}

pub fn left_on(a: Vec2, b: Vec2, c: Vec2) -> bool {
    area(a, b, c) >= 0.0
}

pub fn right(a: Vec2, b: Vec2, c: Vec2) -> bool {
    area(a, b, c) < 0.0
}

pub fn right_on(a: Vec2, b: Vec2, c: Vec2) -> bool {
    area(a, b, c) <= 0.0
}
