use crate::{physics::Physics};

use super::bubble::*;
use super::edge::*;
// use super::vector2::*;

pub fn forcelayout(bubbles: &mut Vec<Bubble>, edges: &mut Vec<Edge>) {
    let time_step = 0.0005;
    let bubble_len = bubbles.len();
    for i in 0..bubble_len {
        let a = &mut bubbles[i].a;
        a.x = 0.0;
        a.y = 0.0;
    }
    for i in 0..(bubble_len - 1) {
        // I need to use unsafe here because rust thinks bubble_a and bubble_b are two borrows from a single object..
        unsafe {
            let imr_a = &mut bubbles[i];
            let bubble_a = imr_a as *mut Bubble;
            for j in (i + 1)..bubble_len {
                let imr_b = &mut bubbles[j];
                let bubble_b = imr_b as *mut Bubble;

                let m_a = (*bubble_a).get_m();
                let m_b = (*bubble_b).get_m();

                // println!("position {:?} {:?}", (*bubble_a).position.clone(), (*bubble_b).position.clone());

                let d_ab = (*bubble_b).position.sub(&(*bubble_a).position);
                let nd_ab = d_ab.norm();
                let repulsive_force_factor = 1.0;
                let repulsive_force = nd_ab.mul_s(repulsive_force_factor * m_a * m_b / d_ab.sqrt_len());

                let a_a = repulsive_force.mul_s(-1.0 / m_a);
                // println!("{:?}", a_a);
                (*bubble_a).a = (*bubble_a).a.add(&a_a);

                let a_b = a_a.mul_s(-1.0 * m_a / m_b);
                (*bubble_b).a = (*bubble_b).a.add(&a_b);
            }
        }
    }

    let edge_len = edges.len();

    for i in 0..edge_len {
        let edge = &mut edges[i];
        let bubble_from = (& mut bubbles[edge.from]) as *mut Bubble;
        let bubble_to = (& mut bubbles[edge.to]) as *mut Bubble;
        unsafe {
            let m_from = (*bubble_from).get_m();
            let m_to = (*bubble_to).get_m();

            let d_from_to = (*bubble_to).position.sub(&(*bubble_from).position);
            let pull_force_factor = 1000000.0;
            let pull_force_from_to = d_from_to.mul_s(pull_force_factor);
            edge.pull_force = pull_force_from_to.len();
            // println!("{}", edge.pull_force);
            let a_from = pull_force_from_to.mul_s(1.0/m_from);
            (*bubble_from).a = (*bubble_from).a.add(&a_from);
            let a_to = a_from.mul_s(-1.0 * m_from / m_to);
            (*bubble_to).a = (*bubble_to).a.add(&a_to);
        }
    }


    for bubble in bubbles.iter_mut() {
        // damping
        bubble.v = bubble.v.mul_s(0.5);

        bubble.v = bubble.v.add(&bubble.a.mul_s(time_step));
        bubble.position = bubble.position.add(&bubble.v.mul_s(time_step));
    }

    for edge in edges {
        edge.position_from.set(& (&bubbles[edge.from]).position);
        edge.position_to.set(& (&bubbles[edge.to]).position);
    }
}
