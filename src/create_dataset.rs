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
    let mut bubbles: Vec<Bubble> = (0..bubble_count)
        .map(|_| Bubble {
            position: get_random_vec2().mul_s(100.0),
            size: rand::random::<f32>() * 20.0 + 5.0,
            // size: 30.0,
            v: Vector2{x: 0.0, y: 0.0},
            a: Vector2{x: 0.0, y: 0.0},
            mesh: Mesh::default(),
        })
        .collect();
    // bubbles[0].size = 40.0;
    bubbles
}

pub fn create_edges(bubble_count: usize, group_size: usize) -> Vec<Edge> {
    let mut edges = vec![];
    // let len = bubbles.len();
    // let first_bubble = &mut bubbles[0];
    // let first_bubble = &mut ((bubbles.split_at_mut(0)).1[0]);
    // let (a,b) = bubbles.split_at_mut(0);
    // // let (c,d) = bubbles.split_at_mut(1);
    // let a1 = &mut a[0];
    // let first_bubble = a1;
    // let b1 = &mut b[0];
    // a1.a.x = 1.0;
    // b1.a.x = 2.0;
    // let fb_rptr = first_bubble as *mut Bubble;
    // for i in 1..len {
    //     let (a,b) = bubbles.split_at_mut(0);
    //     // let (c,d) = bubbles.split_at_mut(1);
    //     let a1 = &mut a[0];
    //     let first_bubble = a1;
    //     // let b_rptr = (&mut bubbles[i]) as *mut Bubble;
    //     let (a,b) = bubbles.split_at_mut(i);
    //     let a1 = &mut a[0];
    //     let second_bubble = a1;
    //     // let b1 = &mut b[0];
    //     // let mut second_bubble = &mut ((bubbles.split_at_mut(i)).1[0]);
    //     // unsafe {
    //         let edge: Edge = Edge {
    //             position: get_random_vec2(),
    //             from: first_bubble,
    //             to: second_bubble
    //         };
    //         edges.push(edge);
    //     // }
    // }
    // for i in 1..bubble_count {
    //     edges.push(Edge {
    //         position_from: get_random_vec2(),
    //         position_to: get_random_vec2(),
    //         from: 0,
    //         to: i,
    //         pull_force: 0.0,
    //     })
    // }
    // edges

    // let group_size = 5;
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
