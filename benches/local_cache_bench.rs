use criterion::{
    criterion_group,
    criterion_main,
    Criterion,
    BenchmarkId
};

use tokio::runtime::Runtime;
use std::sync::Arc;
use rust_kvs::simple_cache::SimpleCache as SimpleCache;
use rust_kvs::Cache;

fn generate_data(count: usize) -> Vec<(String, String)> {
    (0..count)
        .map(|i| (format!("key_{}",i), format!("val_{}", i)))
        .collect()
}

fn bench_reads(c: &mut Criterion) {
    let rt = Runtime::new().unwrap();
    let cache = SimpleCache::<String, String>::new();
    let data = generate_data(1000);

    rt.block_on(async {
        for (k,v) in &data {
            cache.set(k.clone(), v.clone()).await;
        }
    });

    let cache = Arc::new(cache);

    c.bench_with_input(
        BenchmarkId::new("simple_cache_read", "1k_items"),
        &data,
        |b, _data| {
            let cache_ref = cache.clone();
            let key = "key_500".to_string();

            b.to_async(&rt).iter(|| async {
                let _ = cache_ref.get(&key).await;
            })
        }
    );
}

fn bench_mixed_workload(c: &mut Criterion) {
    let rt = Runtime::new().unwrap();
    let cache = Arc::new(SimpleCache::<String, String>::new());

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
                            if i%10 == 0 {
                                let k = format!("key_{}", i);
                                c_clone.set(k, "new_val".into())
                                    .await;
                            } else {
                                let k = format!("key_{}", i);
                                let _ = c_clone.get(&k).await;
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