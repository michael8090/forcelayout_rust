use crate::forcelayout_graph::Bubble;
use crate::forcelayout_graph::BubbleElement;
use crate::forcelayout_graph::Edge;
use crate::forcelayout_graph::EdgeElement;
use crate::graph::Graph;
use crate::graph::Node;
use crate::mesh::Mesh;
use std::cell::RefCell;
use std::fs::File;
use std::io::Read;
use std::rc::Rc;
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

pub fn create_bubbles(bubble_count: u64) -> Vec<Rc<RefCell<Bubble>>> {
    let mut bubbles = (0..bubble_count)
        .map(|_| Bubble::new(BubbleElement {
            position: get_random_vec2().add_s(-0.5).mul_s(100.0),
            size: rand::random::<f32>() * 24.0 + 1.0,
            // size: 100.0,
            v: Vector2{x: 0.0, y: 0.0},
            a: Vector2{x: 0.0, y: 0.0},
            meshes: [Mesh::default(), Mesh::default(), Mesh::default()],
            label: String::from(""),
        }))
        .collect();
    // bubbles[0].position = Vector2{x: 0.0, y: 0.0};
    // bubbles[1].position = Vector2{x: 1.0, y: 0.0};
    bubbles
}

pub fn create_edges(bubbles: Vec<Rc<RefCell<Node<BubbleElement, EdgeElement>>>>, group_size: usize) -> Vec<Rc<RefCell<Edge>>> {
    let mut edges = vec![];
    let bubble_count = bubbles.len();
    let group_count = ((bubble_count as f32) / (group_size as f32)).ceil() as usize;
    for i in 0..group_count {
        let group_item_count = (bubble_count - i * group_size).min(group_size);
        for j in 1..group_item_count {
            let from = bubbles[0 + i*group_size].clone();
            let to = bubbles[j + i*group_size].clone();
            edges.push(Edge::new(&from, &to, EdgeElement {
                pull_force: 0.0,
                mesh: Mesh::default(),
            }))
        }
    }
    edges
}

pub fn create_dataset_from_file() -> Result<(Vec<Rc<RefCell<Bubble>>>, Vec<Rc<RefCell<Edge>>>)> {
    // let mut file = File::open("datasets/miserables.json").unwrap();
    let bytes = include_bytes!("datasets/miserables.json");
    let data = String::from_utf8_lossy(bytes).into_owned();
    // file.read_to_string(&mut data).unwrap();
    let v: Value = serde_json::from_str(&data)?;

    let nodes =  v["nodes"].as_array().unwrap();

    let node_names: Vec<&str> = nodes.into_iter().map(|node| {
        node["id"].as_str().unwrap()
    }).collect();

    let mut bubbles: Vec<Rc<RefCell<Bubble>>> = nodes.into_iter().map(|node| Bubble::new(BubbleElement {
        position: get_random_vec2().add_s(-0.5).mul_s(100.0),
        size: 100.0,
        // size: 100.0,
        v: Vector2{x: 0.0, y: 0.0},
        a: Vector2{x: 0.0, y: 0.0},
        meshes: [Mesh::default(), Mesh::default(), Mesh::default()],
        label: String::from(node["id"].as_str().unwrap()),
    }))
    .collect();

    let links = v["links"].as_array().unwrap();
    
    let edges: Vec<Rc<RefCell<Edge>>> = links.into_iter().map(|link| {
        let source = link["source"].as_str().unwrap();
        let from_index = (&node_names).into_iter().position(|&name| name == source).unwrap();
        // {
        let from_bubble = &mut bubbles[from_index].clone();
        from_bubble.borrow_mut().element.size += 10.0;
        // }

        let target = link["target"].as_str().unwrap();
        let to_index = (&node_names).into_iter().position(|&name| name == target).unwrap();
        
        // {
        let to_bubble = &mut bubbles[to_index].clone();
        to_bubble.borrow_mut().element.size += 10.0;
        // }

        Edge::new(from_bubble, to_bubble, EdgeElement {
            pull_force: 0.0,
            mesh: Mesh::default(),
        })
    })
    .collect();

    Ok((bubbles, edges))
}
