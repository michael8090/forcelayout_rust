use std::borrow::BorrowMut;
use std::cell::RefCell;
use std::f32::consts;
use std::rc::Rc;

use crate::forcelayout_graph::{Bubble, Edge};
use crate::{physics::Physics};

// use super::bubble::*;
// use super::edge::*;
// use super::vector2::*;

pub fn forcelayout(bubbles: &Vec<Rc<RefCell<Bubble>>>, edges: &Vec<Rc<RefCell<Edge>>>) {
    let time_step = 0.5;
    let bubble_len = bubbles.len();
    for i in 0..bubble_len {
        // let x = bubbles[i];
        // let y = x.borrow_mut();
        // let z = y.borrow_mut();

        // cannot use borrow_mut here, why?
        let a = bubbles[i].get_mut().element.a;
        a.x = 0.0;
        a.y = 0.0;
    }
    for i in 0..(bubble_len - 1) {
        // I need to use unsafe here because rust thinks bubble_a and bubble_b are two borrows from a single object..
        // unsafe {
        //     let imr_a = &mut bubbles[i];
        //     let bubble_a = imr_a as *mut Bubble;
        //     for j in (i + 1)..bubble_len {
        //         let imr_b = &mut bubbles[j];
        //         let bubble_b = imr_b as *mut Bubble;

        //         let m_a = (*bubble_a).get_m();
        //         let m_b = (*bubble_b).get_m();

        //         let d_ab = (*bubble_b).position.sub(&(*bubble_a).position);
        //         let nd_ab = d_ab.norm();
        //         let repulsive_force_factor = 1.0;
        //         let repulsive_force = nd_ab.mul_s(repulsive_force_factor * m_a * m_b / d_ab.sqrt_len());

        //         let a_a = repulsive_force.mul_s(-1.0 / m_a);
        //         (*bubble_a).a = (*bubble_a).a.add(&a_a);

        //         let a_b = a_a.mul_s(-1.0 * m_a / m_b);
        //         (*bubble_b).a = (*bubble_b).a.add(&a_b);
        //     }
        // }
        let bubble_a = bubbles[i].get_mut();
        // let bubble_a = imr_a as *mut Bubble;
        for j in (i + 1)..bubble_len {
            let bubble_b = bubbles[j].get_mut();
            // let bubble_b = imr_b as *mut Bubble;

            let m_a = bubble_a.get_m();
            let m_b = bubble_b.get_m();

            let d_ab = bubble_b.element.position.sub(&(bubble_a.element.position));
            let nd_ab = d_ab.norm();
            let repulsive_force_factor = 1.0;
            let repulsive_force = nd_ab.mul_s(repulsive_force_factor * m_a * m_b / d_ab.sqrt_len());

            let a_a = repulsive_force.mul_s(-1.0 / m_a);
            bubble_a.element.a = bubble_a.element.a.add(&a_a);

            let a_b = a_a.mul_s(-1.0 * m_a / m_b);
            bubble_b.element.a = bubble_b.element.a.add(&a_b);
        }
    }

    let edge_len = edges.len();

    for i in 0..edge_len {
        let edge = &mut edges[i].get_mut();
        let bubble_from = edge.from.get_mut();
        let bubble_to = edge.to.get_mut();
        // unsafe {
            let m_from = bubble_from.get_m();
            let m_to = bubble_to.get_m();

            let d_from_to = bubble_to.element.position.sub(&bubble_from.element.position);
            let pull_force_factor = 1.0;
            let pull_force_from_to = d_from_to.mul_s(pull_force_factor);
            edge.element.pull_force = pull_force_from_to.len();
            let a_from = pull_force_from_to.mul_s(1.0/m_from);
            bubble_from.element.a = bubble_from.element.a.add(&a_from);
            let a_to = a_from.mul_s(-1.0 * m_from / m_to);
            bubble_to.element.a = bubble_to.element.a.add(&a_to);
        // }
    }


    for bubble in bubbles.iter_mut() {
        // `element` doesn't have a mut notation, why is it writable?
        let element = bubble.get_mut().element;
        element.v = element.v.add(&element.a.mul_s(time_step));

        // damping, the higher the velocity is, the quicker it damps
        element.v = element.v.mul_s((1.0 - (element.v.len() * 0.1).atan() *2.0 / std::f32::consts::PI).min(0.9));

        element.position = element.position.add(&element.v.mul_s(time_step));
    }
}
