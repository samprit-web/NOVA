// Benchmarks for NOVA Compiler and Standard Library

use criterion::{black_box, criterion_group, criterion_main, Criterion, BenchmarkId};
use nova_std::collections::*;
use nova_std::string::*;
use nova_std::io::*;

fn bench_vec_push(c: &mut Criterion) {
    let mut group = c.benchmark_group("vec_push");
    
    for size in [10, 100, 1000, 10000].iter() {
        group.bench_with_input(BenchmarkId::from_parameter(size), size, |b, &size| {
            b.iter(|| {
                let mut v = Vec::with_capacity(size);
                for i in 0..size {
                    v.push(black_box(i));
                }
                black_box(v);
            });
        });
    }
    group.finish();
}

fn bench_vec_iter(c: &mut Criterion) {
    let mut group = c.benchmark_group("vec_iter");
    
    for size in [10, 100, 1000, 10000].iter() {
        let vec: Vec<i32> = (0..*size).collect();
        
        group.bench_with_input(BenchmarkId::from_parameter(size), size, |b, _| {
            b.iter(|| {
                let mut sum = 0;
                for x in &vec {
                    sum += black_box(*x);
                }
                black_box(sum);
            });
        });
    }
    group.finish();
}

fn bench_hashmap_insert(c: &mut Criterion) {
    let mut group = c.benchmark_group("hashmap_insert");
    
    for size in [10, 100, 1000, 10000].iter() {
        group.bench_with_input(BenchmarkId::from_parameter(size), size, |b, &size| {
            b.iter(|| {
                let mut m = HashMap::with_capacity(size);
                for i in 0..size {
                    m.insert(black_box(i), black_box(i * 2));
                }
                black_box(m);
            });
        });
    }
    group.finish();
}

fn bench_hashmap_lookup(c: &mut Criterion) {
    let mut group = c.benchmark_group("hashmap_lookup");
    
    for size in [10, 100, 1000, 10000].iter() {
        let mut m = HashMap::with_capacity(*size);
        for i in 0..*size {
            m.insert(i, i * 2);
        }
        
        group.bench_with_input(BenchmarkId::from_parameter(size), size, |b, _| {
            b.iter(|| {
                let mut sum = 0;
                for i in 0..*size {
                    sum += m.get(&black_box(i)).copied().unwrap_or(0);
                }
                black_box(sum);
            });
        });
    }
    group.finish();
}

fn bench_string_operations(c: &mut Criterion) {
    let mut group = c.benchmark_group("string_ops");
    
    group.bench_function("push_str", |b| {
        b.iter(|| {
            let mut s = String::new();
            for _ in 0..100 {
                s.push_str(black_box("hello "));
            }
            black_box(s);
        });
    });
    
    group.bench_function("format", |b| {
        b.iter(|| {
            let s = format!("{} {} {}", black_box(42), black_box("hello"), black_box(3.14));
            black_box(s);
        });
    });
    
    group.bench_function("concat", |b| {
        let a = "hello";
        let b = "world";
        b.iter(|| {
            let s = a.to_string() + b;
            black_box(s);
        });
    });
    
    group.finish();
}

fn bench_io_operations(c: &mut Criterion) {
    let mut group = c.benchmark_group("io_ops");
    
    let data = vec![0u8; 10000];
    
    group.bench_function("buf_reader", |b| {
        b.iter(|| {
            let mut reader = BufReader::new(&data[..]);
            let mut buf = vec![0u8; 1024];
            while reader.read(&mut buf).unwrap() > 0 {}
        });
    });
    
    group.bench_function("cursor_read", |b| {
        b.iter(|| {
            let mut cursor = Cursor::new(&data[..]);
            let mut buf = vec![0u8; 1024];
            while cursor.read(&mut buf).unwrap() > 0 {}
        });
    });
    
    group.finish();
}

fn bench_collections_mixed(c: &mut Criterion) {
    let mut group = c.benchmark_group("collections_mixed");
    
    group.bench_function("vec_filter_map", |b| {
        let data: Vec<i32> = (0..10000).collect();
        b.iter(|| {
            let result: Vec<i32> = data.iter()
                .filter(|x| *x % 2 == 0)
                .map(|x| x * 2)
                .collect();
            black_box(result);
        });
    });
    
    group.bench_function("hashmap_group_by", |b| {
        let data: Vec<i32> = (0..10000).collect();
        b.iter(|| {
            let mut groups = HashMap::new();
            for x in &data {
                *groups.entry(x % 100).or_insert(0) += 1;
            }
            black_box(groups);
        });
    });
    
    group.finish();
}

criterion_group!(
    benches,
    bench_vec_push,
    bench_vec_iter,
    bench_hashmap_insert,
    bench_hashmap_lookup,
    bench_string_operations,
    bench_io_operations,
    bench_collections_mixed
);

criterion_main!(benches);