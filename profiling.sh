#!/bin/bash

if [ "$#" -lt 1 ]; then
    echo "Usage: $0 <workload> [pattern]"
    echo "Workloads: load_factor, key_distribution, operation_mix"
    echo "Patterns for key_distribution: uniform, clustered, sequential"
    echo "Patterns for operation_mix: read_heavy, write_heavy, balanced, typical_web"
    exit 1
fi

WORKLOAD=$1
PATTERN=$2
RESULTS_DIR="profiling_results/${WORKLOAD}"
[ ! -z "$PATTERN" ] && RESULTS_DIR="${RESULTS_DIR}_${PATTERN}"

# Create results directory
mkdir -p "$RESULTS_DIR"

# Function to run profiling for a specific implementation
profile_impl() {
    local impl=$1
    echo "Profiling $impl implementation..."
    
    # Base command
    local cmd="./target/release/hashmap -w $WORKLOAD -i $impl"
    [ ! -z "$PATTERN" ] && {
        if [ "$WORKLOAD" = "key_distribution" ]; then
            cmd="$cmd -k $PATTERN"
        elif [ "$WORKLOAD" = "operation_mix" ]; then
            cmd="$cmd -o $PATTERN"
        fi
    }
    
    # Cache statistics
    echo "Collecting cache statistics for ${impl}..."
    perf stat -e cache-references,cache-misses,L1-dcache-loads,L1-dcache-load-misses,LLC-loads,LLC-load-misses \
    $cmd 2> "$RESULTS_DIR/${impl}_cache.txt"

    # Generate flamegraph
    echo "Generating flamegraph for ${impl}..."
    perf record -F 9973 -g $cmd
    perf script | stackcollapse-perf.pl | flamegraph.pl > "$RESULTS_DIR/${impl}_flame.svg"

    # CPU metrics
    echo "Collecting CPU metrics for ${impl}..."
    perf stat -e context-switches,cpu-migrations,cycles,instructions,branches,branch-misses,cpu-clock,task-clock \
    $cmd 2> "$RESULTS_DIR/${impl}_cpu.txt"
}

# Build in release mode
cargo build --release

# Run for both implementations
profile_impl "chaining"
profile_impl "open_addressing"

echo "Done! Results are in $RESULTS_DIR/"