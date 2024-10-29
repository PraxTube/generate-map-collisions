use bevy::prelude::*;

use crate::point::{at, left, left_on, right, right_on};

pub const ATOL: f32 = 1e-08;

fn make_ccw(poly: &mut Vec<Vec2>) {
    let mut br = 0;

    for i in 1..poly.len() {
        if poly[i].y < poly[br].y || (poly[i].y == poly[br].y && poly[i].x > poly[br].x) {
            br = i;
        }
    }

    if !left(
        at(poly, br as i32 - 1),
        at(poly, br as i32),
        at(poly, br as i32 + 1),
    ) {
        poly.reverse();
    }
}

fn is_reflex(poly: &Vec<Vec2>, i: i32) -> bool {
    right(at(poly, i - 1), at(poly, i), at(poly, i + 1))
}

fn intersection(p1: Vec2, p2: Vec2, q1: Vec2, q2: Vec2) -> Vec2 {
    let a1 = p2.y - p1.y;
    let a2 = q2.y - q1.y;

    let b1 = p1.x - p2.x;
    let b2 = q1.x - q2.x;

    let c1 = a1 * p1.x + b1 * p1.y;
    let c2 = a2 * q1.x + b2 * q1.y;

    let det = a1 * b2 - a2 * b1;

    // Lines are not parallel
    if det.abs() > ATOL {
        return Vec2::new(b2 * c1 - b1 * c2, a1 * c2 - a2 * c1) / det;
    }
    Vec2::default()
}

pub fn decompose_poly(poly: &mut Vec<Vec2>) -> Vec<Vec<Vec2>> {
    assert!(poly.len() > 2, "Length of given poly is < 3, {:?}", poly);

    make_ccw(poly);

    let mut upper_inter = Vec2::default();
    let mut lower_inter = Vec2::default();
    let mut p;

    let mut d;
    let mut closest_dist;

    let mut upper_index = 0;
    let mut lower_index = 0;
    let mut closest_index = 0;

    for i in 0..poly.len() as i32 {
        if !is_reflex(poly, i) {
            continue;
        }

        let mut lower_poly = Vec::new();
        let mut upper_poly = Vec::new();

        let mut upper_dist = f32::MAX;
        let mut lower_dist = f32::MAX;

        for j in 0..poly.len() as i32 {
            // Line intersects with an edge
            if left(at(poly, i - 1), at(poly, i), at(poly, j))
                && right_on(at(poly, i - 1), at(poly, i), at(poly, j - 1))
            {
                // Find point of intersection
                p = intersection(at(poly, i - 1), at(poly, i), at(poly, j), at(poly, j - 1));

                // Make sure intersection point is inside the poly
                if right(at(poly, i + 1), at(poly, i), p) {
                    d = at(poly, i).distance_squared(p);
                    if d < lower_dist {
                        // Only keep closest intersection
                        lower_dist = d;
                        lower_inter = p;
                        lower_index = j;
                    }
                }
            }
            if left(at(poly, i + 1), at(poly, i), at(poly, j + 1))
                && right_on(at(poly, i + 1), at(poly, i), at(poly, j))
            {
                p = intersection(at(poly, i + 1), at(poly, i), at(poly, j), at(poly, j + 1));

                if left(at(poly, i - 1), at(poly, i), p) {
                    d = at(poly, i).distance_squared(p);
                    if d < upper_dist {
                        upper_dist = d;
                        upper_inter = p;
                        upper_index = j;
                    }
                }
            }
        }

        // No vertices to connect to, choose a point in the middle
        if lower_index == ((upper_index + 1) % poly.len() as i32) {
            p = (lower_inter + upper_inter) / 2.0;

            if i < upper_index {
                lower_poly.extend_from_slice(&poly[i as usize..=upper_index as usize]);
                lower_poly.push(p);
                upper_poly.push(p);
                if lower_index != 0 {
                    assert!(lower_index > 0);
                    upper_poly.extend_from_slice(&poly[lower_index as usize..poly.len()]);
                }
                upper_poly.extend_from_slice(&poly[0..=i as usize]);
            } else {
                if i != 0 {
                    lower_poly.extend_from_slice(&poly[i as usize..poly.len()]);
                }
                lower_poly.extend_from_slice(&poly[0..=upper_index as usize]);
                lower_poly.push(p);
                upper_poly.push(p);
                assert!(lower_index > 0);
                upper_poly.extend_from_slice(&poly[lower_index as usize..=i as usize]);
            }
        } else {
            if lower_index > upper_index {
                upper_index += poly.len() as i32;
            }

            closest_dist = f32::MAX;
            for j in lower_index..=upper_index {
                if left_on(at(poly, i - 1), at(poly, i), at(poly, j))
                    && right_on(at(poly, i + 1), at(poly, i), at(poly, j))
                {
                    d = at(poly, i).distance_squared(at(poly, j));
                    if d < closest_dist {
                        closest_dist = d;
                        // TODO: Potentially bad maybe?
                        closest_index = j % poly.len() as i32;
                    }
                }
            }

            if i < closest_index {
                assert!(closest_index > 0);
                lower_poly.extend_from_slice(&poly[i as usize..=closest_index as usize]);
                if closest_index != 0 {
                    upper_poly.extend_from_slice(&poly[closest_index as usize..poly.len()]);
                }
                upper_poly.extend_from_slice(&poly[0..=i as usize]);
            } else {
                if i != 0 {
                    lower_poly.extend_from_slice(&poly[i as usize..poly.len()]);
                }
                lower_poly.extend_from_slice(&poly[0..=closest_index as usize]);
                upper_poly.extend_from_slice(&poly[closest_index as usize..=i as usize]);
            }
        }

        let (mut smaller, mut bigger) = if lower_poly.len() < upper_poly.len() {
            (
                decompose_poly(&mut lower_poly),
                decompose_poly(&mut upper_poly),
            )
        } else {
            (
                decompose_poly(&mut upper_poly),
                decompose_poly(&mut lower_poly),
            )
        };
        smaller.append(&mut bigger);
        return smaller;
    }
    vec![poly.clone()]
}
