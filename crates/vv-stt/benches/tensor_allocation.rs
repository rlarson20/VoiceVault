// Benchmark for tensor allocation in Whisper decoder loop
//
// This benchmark measures the performance of the current tensor allocation
// pattern used in decode_greedy. Use this to establish a baseline before
// optimizing the tensor allocation strategy.
//
// Run with: cargo bench --bench tensor_allocation

use candle_core::{Device, Result, Tensor};
use criterion::{black_box, criterion_group, criterion_main, Criterion, BenchmarkId};

/// Simulates the tensor allocation pattern from decode_greedy
/// This is the current implementation approach.
fn decode_greedy_baseline(
    device: &Device,
    max_iterations: usize,
) -> Result<Vec<u32>> {
    // Starting tokens (SOT, transcribe, no_timestamps)
    let mut tokens = vec![50258u32, 50359, 50363];

    for i in 0..max_iterations {
        // This is the pattern we're benchmarking:
        // Creating a new tensor on every iteration
        let tokens_tensor = Tensor::new(tokens.as_slice(), device)?
            .unsqueeze(0)?;

        // Simulate some work (in real code, this is the decoder forward pass)
        // We'll just do some tensor operations to make it realistic
        let _dummy = tokens_tensor.clone();

        // Simulate getting a next token
        let next_token = (i % 1000) as u32;
        tokens.push(next_token);

        // Early exit condition
        if next_token == 50257 || tokens.len() >= max_iterations {
            break;
        }
    }

    Ok(tokens)
}

/// Benchmark different decode lengths (simulating different transcription results)
fn bench_decode_greedy_iterations(c: &mut Criterion) {
    let device = Device::Cpu;

    let mut group = c.benchmark_group("decode_iterations");

    // Test with different iteration counts (simulating different output lengths)
    for iterations in [10, 25, 50, 75, 100].iter() {
        group.bench_with_input(
            BenchmarkId::new("baseline", iterations),
            iterations,
            |b, &iterations| {
                b.iter(|| {
                    black_box(
                        decode_greedy_baseline(&device, iterations).unwrap()
                    )
                })
            },
        );
    }

    group.finish();
}

/// Benchmark the tensor allocation overhead specifically
fn bench_tensor_creation_overhead(c: &mut Criterion) {
    let device = Device::Cpu;

    let mut group = c.benchmark_group("tensor_creation");

    // Benchmark creating tensors of different sizes
    for size in [5, 10, 25, 50, 100].iter() {
        let tokens: Vec<u32> = (0..*size).map(|i| i as u32).collect();

        group.bench_with_input(
            BenchmarkId::new("create_and_unsqueeze", size),
            &tokens,
            |b, tokens| {
                b.iter(|| {
                    let tensor = Tensor::new(black_box(tokens), &device)
                        .unwrap()
                        .unsqueeze(0)
                        .unwrap();
                    black_box(tensor);
                })
            },
        );
    }

    group.finish();
}

/// Benchmark the cumulative allocation pattern (what happens in a full decode)
fn bench_cumulative_allocation(c: &mut Criterion) {
    let device = Device::Cpu;

    let mut group = c.benchmark_group("cumulative_allocation");

    // This simulates a real decode: starting with 3 tokens and growing to ~50
    group.bench_function("realistic_decode_growth", |b| {
        b.iter(|| {
            let mut tokens = vec![50258u32, 50359, 50363]; // SOT, transcribe, no_timestamps
            for new_token in 0..50 {
                // Create new tensor each time (current pattern)
                let _tensor = Tensor::new(tokens.as_slice(), &device)
                    .unwrap()
                    .unsqueeze(0)
                    .unwrap();
                black_box(&_tensor);

                // Add next token
                tokens.push(new_token);
            }
        })
    });

    group.finish();
}

/// Compare allocation patterns with and without the unsqueeze operation
fn bench_tensor_vs_unsqueeze(c: &mut Criterion) {
    let device = Device::Cpu;

    let mut group = c.benchmark_group("tensor_operations");

    let tokens: Vec<u32> = (0..50).collect();

    group.bench_function("tensor_creation_only", |b| {
        b.iter(|| {
            let tensor = Tensor::new(black_box(&tokens), &device).unwrap();
            black_box(tensor);
        })
    });

    group.bench_function("tensor_with_unsqueeze", |b| {
        b.iter(|| {
            let tensor = Tensor::new(black_box(&tokens), &device)
                .unwrap()
                .unsqueeze(0)
                .unwrap();
            black_box(tensor);
        })
    });

    group.finish();
}

criterion_group!(
    name = benches;
    config = Criterion::default().sample_size(50);
    targets = bench_decode_greedy_iterations,
             bench_tensor_creation_overhead,
             bench_cumulative_allocation,
             bench_tensor_vs_unsqueeze
);

criterion_main!(benches);
