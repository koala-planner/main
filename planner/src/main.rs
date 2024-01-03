#![allow(unused)]
use std::env;

mod domain_description;
mod graph_lib;
mod task_network;
mod search;
mod relaxation;
mod heuristic_calculator;
mod visualization;

use domain_description::read_json_domain;
use crate::search::SearchResult;

fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() < 2 {
        println!("The path to the problem file is not given.");
        return;
    }
    let problem = read_json_domain(&args[1]);
    let (solution, stats) = search::AOStarSearch::run(&problem);
    print!("{}", stats);
    match solution {
        SearchResult::Success(x) => {
            println!("makespan: {}", x.makespan);
            println!("***************************");
            println!("{}", x);
        },
        SearchResult::NoSolution => {
            println!("Problem has no solution")
        }
    }
}
