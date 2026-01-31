use criterion::{criterion_group, criterion_main, BenchmarkId, Criterion};
use rkyv::AlignedVec;
use rust_kvs::ByteCache;
use std::sync::Arc;
use tokio::runtime::Runtime;

fn to_aligned_vec(s: &str) -> AlignedVec {
    let mut vec = AlignedVec::new();
    vec.extend_from_slice(s.as_bytes());
    vec
}

fn generate_data(count: usize) -> Vec<(String, String)> {
    (0..count)
        .map(|i| (format!("key_{}", i), format!("val_{}", i)))
        .collect()
}

fn bench_reads(c: &mut Criterion) {
    let rt = Runtime::new().unwrap();
    let cache = ByteCache::<String>::new();
    let data = generate_data(1000);

    for (k, v) in &data {
        let bytes = to_aligned_vec(v);
        cache.set(k.clone(), bytes, None);
    }

    let cache = Arc::new(cache);

    c.bench_with_input(
        BenchmarkId::new("simple_cache_read", "1k_items"),
        &data,
        |b, _data| {
            let cache_ref = cache.clone();
            let key = "key_500".to_string();

            b.to_async(&rt).iter(|| async {
                let _ = cache_ref.get(&key);
            })
        },
    );
}

fn bench_mixed_workload(c: &mut Criterion) {
    let rt = Runtime::new().unwrap();
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
                                let val = to_aligned_vec("new_val");
                                c_clone.set(k, val, None);
                            } else {
                                let _ = c_clone.get(&k);
                            }
                        }));
                    }
                    for h in handles {
                        let _ = h.await;
                    }
                })
            },
        );
    }
    group.finish();
}

criterion_group!(benches, bench_reads, bench_mixed_workload);

criterion_main!(benches);
