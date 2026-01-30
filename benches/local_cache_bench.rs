use criterion::{criterion_group, criterion_main, Criterion, BenchmarkId};
use tokio::runtime::Runtime;
use std::sync::Arc;
// Assuming these are available in your crate root or module
use rust_kvs::bytestore::{ByteCache, to_bytes}; 

fn generate_data(count: usize) -> Vec<(String, String)> {
    (0..count)
        .map(|i| (format!("key_{}", i), format!("val_{}", i)))
        .collect()
}

fn bench_reads(c: &mut Criterion) {
    let rt = Runtime::new().unwrap();
    let cache = ByteCache::<String>::new();
    let data = generate_data(1000);

    // Sync calls: No .await needed
    for (k, v) in &data {
        let bytes = to_bytes(v); // Convert String to AlignedVec
        cache.set(k.clone(), bytes, None); // Added None for TTL
    }

    let cache = Arc::new(cache);

    c.bench_with_input(
        BenchmarkId::new("simple_cache_read", "1k_items"),
        &data,
        |b, _data| {
            let cache_ref = cache.clone();
            let key = "key_500".to_string();

            // iter() can still be async if the bench framework requires it,
            // but the internal call is now sync.
            b.to_async(&rt).iter(|| async {
                let _ = cache_ref.get(&key); // Removed .await
            })
        }
    );
}

fn bench_mixed_workload(c: &mut Criterion) {
    let rt = Runtime::new().unwrap();
    // Fixed: Only one generic argument <String>
    let cache = Arc::new(ByteCache::<String>::new());

    let mut group = c.benchmark_group("contention");

    for concurrency in [10, 50, 100].iter() {
        group.bench_with_input(
            BenchmarkId::from_parameter(concurrency),
            concurrency,
            |b, &size| {
                b.to_async(&rt).iter(|| async {
                    let mut handles = Vec::new();
                    for i in 0..size {
                        let c_clone = cache.clone();
                        handles.push(tokio::spawn(async move {
                            let k = format!("key_{}", i);
                            if i % 10 == 0 {
                                let val = to_bytes(&"new_val".to_string());
                                // Removed .await and added None
                                c_clone.set(k, val, None);
                            } else {
                                // Removed .await
                                let _ = c_clone.get(&k);
                            }
                        }));
                    }
                    for h in handles {
                        let _ = h.await;
                    }
                })
            }
        );
    }
    group.finish();
}

criterion_group!(
    benches,
    bench_reads,
    bench_mixed_workload
);

criterion_main!(benches);