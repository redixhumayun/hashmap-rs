#![feature(trait_alias)]

use clap::Parser;

mod chaining;
mod open_addressing;
mod workloads;

use hashmap::workloads::{generators::run_load_factor_workload, LoadFactorWorkload};
use workloads::{
    generators::{run_key_distribution_workload, run_operation_mix_workload},
    KeyDistributionWorkload, OperationMixWorkload,
};

#[derive(Parser, Debug)]
struct Args {
    #[arg(short, long)]
    workload: String,

    #[arg(short, long)]
    implementation: String,

    #[arg(short, long)]
    #[arg(requires = "workload")]
    #[arg(required_if_eq("workload", "key_distribution"))]
    key_dist: Option<String>,

    #[arg(short, long)]
    #[arg(requires = "workload")]
    #[arg(required_if_eq("workload", "operation_mix"))]
    op_mix: Option<String>,
}

fn main() {
    let args = Args::parse();

    match args.workload.as_str() {
        "load_factor" => match args.implementation.as_str() {
            "chaining" => run_load_factor_workload(&LoadFactorWorkload {
                size: 1_000_000,
                value_size: 100,
            }),
            "open_addressing" => run_load_factor_workload(&LoadFactorWorkload {
                size: 1_000_000,
                value_size: 100,
            }),
            _ => eprintln!("invalid implementation"),
        },
        "key_distribution" => {
            let pattern = match args.key_dist.as_deref() {
                Some("uniform") => workloads::KeyPattern::Uniform,
                Some("clustered") => workloads::KeyPattern::Clustered,
                Some("sequential") => workloads::KeyPattern::Sequential,
                _ => {
                    panic!("Invalid key distribution pattern");
                    return;
                }
            };

            match args.implementation.as_str() {
                "chaining" => run_key_distribution_workload(&KeyDistributionWorkload {
                    size: 1000,
                    pattern,
                }),
                "open_addressing" => run_key_distribution_workload(&KeyDistributionWorkload {
                    size: 1000,
                    pattern,
                }),
                _ => panic!("invalid implementation"),
            }
        }
        "operation_mix" => {
            let (read_pct, write_pct) = match args.op_mix.as_deref() {
                Some("read_heavy") => (90, 5),
                Some("write_heavy") => (5, 90),
                Some("balanced") => (33, 33),
                Some("typica_web") => (80, 15),
                _ => {
                    panic!("Invalid operation mix pattern")
                }
            };

            match args.implementation.as_str() {
                "chaining" => run_operation_mix_workload(&OperationMixWorkload {
                    initial_size: 1000,
                    operations: 1000,
                    read_pct,
                    write_pct,
                }),
                "open_addressing" => run_operation_mix_workload(&OperationMixWorkload {
                    initial_size: 1000,
                    operations: 1000,
                    read_pct,
                    write_pct,
                }),
                _ => panic!("invalid implementation"),
            }
        }
        _ => panic!("Invalid workload"),
    }
}
