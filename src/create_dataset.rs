use crate::mesh::Mesh;
use std::fs::File;
use std::io::Read;
use serde_json::{Result, Value};

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
    let mut bubbles: Vec<Bubble> = (0..bubble_count)
        .map(|_| Bubble {
            position: get_random_vec2().add_s(-0.5).mul_s(100.0),
            size: rand::random::<f32>() * 24.0 + 1.0,
            // size: 100.0,
            v: Vector2{x: 0.0, y: 0.0},
            a: Vector2{x: 0.0, y: 0.0},
            meshes: [Mesh::default(), Mesh::default(), Mesh::default()],
        })
        .collect();
    // bubbles[0].position = Vector2{x: 0.0, y: 0.0};
    // bubbles[1].position = Vector2{x: 1.0, y: 0.0};
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

pub fn create_dataset_from_file() -> Result<(Vec<Bubble>, Vec<Edge>)> {
    // let mut file = File::open("datasets/miserables.json").unwrap();
    let bytes = include_bytes!("datasets/miserables.json");
    let data = String::from_utf8_lossy(bytes).into_owned();
    // file.read_to_string(&mut data).unwrap();
    let v: Value = serde_json::from_str(&data)?;

    let nodes =  v["nodes"].as_array().unwrap();

    let node_names: Vec<&str> = nodes.into_iter().map(|node| {
        node["id"].as_str().unwrap()
    }).collect();

    let bubbles = nodes.into_iter().map(|_| Bubble {
        position: get_random_vec2().add_s(-0.5).mul_s(100.0),
        size: rand::random::<f32>() * 24.0 + 1.0,
        // size: 100.0,
        v: Vector2{x: 0.0, y: 0.0},
        a: Vector2{x: 0.0, y: 0.0},
        meshes: [Mesh::default(), Mesh::default(), Mesh::default()],
    })
    .collect();

    let links = v["links"].as_array().unwrap();
    
    let edges = links.into_iter().map(|link| {
        let source = link["source"].as_str().unwrap();
        let from = (&node_names).into_iter().position(|&name| name == source).unwrap();

        let target = link["target"].as_str().unwrap();
        let to = (&node_names).into_iter().position(|&name| name == target).unwrap();

        Edge {
            position_from: get_random_vec2(),
            position_to: get_random_vec2(),
            from,
            to,
            pull_force: 0.0,
            mesh: Mesh::default(),
        }
    })
    .collect();

    Ok((bubbles, edges))
}
