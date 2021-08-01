use std::{cell::RefCell, rc::Rc};

use crate::id_generator;

pub struct Graph<T, U> {
    nodes: Vec<Rc<RefCell<Node<T, U>>>>,
    edges: Vec<Rc<RefCell<Edge<T, U>>>>,
}

pub struct Node<T, U> {
    pub id: i32,
    pub element: T,
    pub in_edges: Vec<Rc<RefCell<Edge<T, U>>>>,
    pub out_edges: Vec<Rc<RefCell<Edge<T, U>>>>,
}

impl<T, U> Node<T, U> {
    pub fn new(element: T) -> Rc<RefCell<Self>>  {
        id_generator::ID_GENERATOR.with(|ig| {
            Rc::new(RefCell::new(Node {
                id: ig.get_mut().get(),
                element,
                in_edges: vec![],
                out_edges: vec![],
            }))
        })
    }
}

pub struct Edge<T, U> {
    pub id: i32,
    pub element: U,
    pub from: Rc<RefCell<Node<T, U>>>,
    pub to: Rc<RefCell<Node<T, U>>>,
}

impl<T, U> Edge<T, U> {
    pub fn new(from: &Rc<RefCell<Node<T, U>>>, to: &Rc<RefCell<Node<T, U>>>, element: U) -> Rc<RefCell<Self>>  {
        id_generator::ID_GENERATOR.with(|ig| {
            Rc::new(RefCell::new(Edge {
                id: ig.get_mut().get(),
                element,
                from: from.clone(),
                to: to.clone(),
            }))
        })
    }
}

// fn create() {
//     let mut graph = Graph {
//         edges: vec![],
//         nodes: vec![],
//     };
//     let node = Node::new(1.0);
//     let edge = Edge::new(&node, &node, "a");

//     // edge.get_mut().from.get_mut().value = 2.0;
//     // edge.get_mut().value = "b";
//     edge.borrow_mut().value = "b";
//     edge.borrow().from.borrow_mut().value = 2.0;


//     graph.edges.push(edge);
//     graph.nodes.push(node);
// }
