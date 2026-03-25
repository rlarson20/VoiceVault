# Tensor Allocation Benchmarks

This directory contains benchmarks for measuring tensor allocation performance in the Whisper decoder.

## Running the Benchmarks

```bash
# Run all benchmarks
cargo bench --bench tensor_allocation

# Run specific benchmark group
cargo bench --bench tensor_allocation -- decode_iterations

# Run with more iterations for more stable results
cargo bench --bench tensor_allocation -- --sample-size 100

# Save benchmark results for comparison
cargo bench --bench tensor_allocation -- --save-baseline before_optimization

# Compare against saved baseline
cargo bench --bench tensor_allocation -- --baseline before_optimization
```

## Benchmark Groups

### `decode_iterations`
Measures performance of the full decode loop with different iteration counts. This simulates transcriptions of varying lengths (short vs long audio).

### `tensor_creation_overhead`
Isolates the tensor creation and unsqueeze operation cost. Shows how expensive the allocation is for different tensor sizes.

### `cumulative_allocation`
Simulates a realistic decode cycle where tokens grow from 3 to ~50. This shows the cumulative effect of repeated allocations.

### `tensor_vs_unsqueeze`
Compares tensor creation with and without the unsqueeze operation to understand the overhead of adding the batch dimension.

## Using Benchmarks for Optimization

1. **Establish baseline** before optimization:
   ```bash
   cargo bench --bench tensor_allocation -- --save-baseline before
   ```

2. **Implement your optimization** in the engine

3. **Compare results**:
   ```bash
   cargo bench --bench tensor_allocation -- --baseline before
   ```

4. **Iterate** based on the comparison data

## Expected Results

Before optimization, you should see:
- Linear growth in time with iteration count
- Significant time spent on tensor allocation
- Each iteration doing 2+ allocations (tensor + unsqueeze)

After optimization, you should see:
- Reduced allocation overhead
- Better cache locality
- More consistent timing across iterations
