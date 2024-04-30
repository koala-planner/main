#![allow(unused)]
use std::env;

extern crate bit_vec;

mod domain_description;
mod graph_lib;
mod task_network;
mod search;
mod relaxation;
mod heuristics;

use domain_description::read_json_domain;
use crate::search::{SearchResult, HeuristicType};

fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() < 2 {
        println!("The path to the problem file is not given.");
        return;
    }
    let problem = read_json_domain(&args[1]);
    let (solution, stats) = search::AOStarSearch::run(&problem, HeuristicType::HAdd);
    print!("{}", stats);
    match solution {
        SearchResult::Success(x) => {
            println!("makespan: {}", x.makespan);
            println!("policy enteries: {}", x.transitions.len());
            //println!("***************************");
            //println!("{}", x);
        },
        SearchResult::NoSolution => {
            println!("Problem has no solution")
        }
    }
}
