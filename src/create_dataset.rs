use crate::mesh::Mesh;

use super::bubble::*;
use super::edge::*;
use super::math::*;

fn get_random_vec2() -> Vector2 {
    Vector2 {
        x: rand::random(),
        y: rand::random(),
    }
}

pub fn create_bubbles(bubble_count: u64) -> Vec<Bubble> {
    let bubbles: Vec<Bubble> = (0..bubble_count)
        .map(|_| Bubble {
            position: get_random_vec2().add_s(-0.5).mul_s(100.0),
            size: rand::random::<f32>() * 20.0 + 5.0,
            // size: 30.0,
            v: Vector2{x: 0.0, y: 0.0},
            a: Vector2{x: 0.0, y: 0.0},
            meshes: [Mesh::default(), Mesh::default(), Mesh::default()],
        })
        .collect();
    // bubbles[0].size = 40.0;
    bubbles
}

pub fn create_edges(bubble_count: usize, group_size: usize) -> Vec<Edge> {
    let mut edges = vec![];
    let group_count = ((bubble_count as f32) / (group_size as f32)).ceil() as usize;
    for i in 0..group_count {
        let group_item_count = (bubble_count - i * group_size).min(group_size);
        for j in 1..group_item_count {
            edges.push(Edge {
                position_from: get_random_vec2(),
                position_to: get_random_vec2(),
                from: 0 + i*group_size,
                to: j + i*group_size,
                pull_force: 0.0,
                mesh: Mesh::default(),
            })
        }
    }
    edges
}
