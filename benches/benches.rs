#![allow(dead_code)]
use std::alloc::Layout;
use std::mem::size_of;
use criterion::*;
use scratch::{BumpDown, BumpUp};

#[derive(Default, Hash)]
struct Small(u8);

#[derive(Default, Hash)]
struct Medium(usize);

#[derive(Default, Hash)]
struct Big(usize, usize);

impl From<usize> for Big {
    fn from(val: usize) -> Self {
        Self(val, val)
    }
}

impl From<usize> for Medium {
    fn from(val: usize) -> Self {
        Self(val)
    }
}

impl From<usize> for Small {
    fn from(val: usize) -> Self {
        Self(val as u8)
    }
}

trait Alloc {
    fn alloc(&mut self, layout: Layout) -> Option<std::ptr::NonNull<u8>>;
    fn clear(&mut self) {}
}

impl Alloc for BumpUp {
    fn alloc(&mut self, layout: Layout) -> Option<std::ptr::NonNull<u8>> {
        self.alloc(layout)
    }
    fn clear(&mut self) {
        self.clear();
    }
}

impl Alloc for BumpDown {
    fn alloc(&mut self, layout: Layout) -> Option<std::ptr::NonNull<u8>> {
        self.alloc_aligned(layout)
    }
    fn clear(&mut self) {
        self.clear();
    }
}


fn alloc<T: Default + From<usize>, A: Alloc>(arena: &mut A, n: usize) {
    let layout = Layout::new::<T>();
    arena.clear();
    for i in 0..n {
        let ptr = arena.alloc(layout).unwrap();
        unsafe {ptr.as_ptr().cast::<T>().write(T::default())}
    }
}

const ALLOCATIONS: usize = 10000;

fn bench_alloc(c: &mut Criterion) {
    let mut group = c.benchmark_group("alloc");
    type T = Medium;
    let mut arena = BumpDown::with_capacity(ALLOCATIONS * size_of::<T>());
    group.bench_function("down", |b| b.iter(|| alloc::<T, BumpDown>(&mut arena, ALLOCATIONS)));
    let mut arena = BumpUp::with_capacity(ALLOCATIONS * size_of::<T>());
    group.bench_function("up", |b| b.iter(|| alloc::<T, BumpUp>(&mut arena, ALLOCATIONS)));
}

criterion_group!(
    benches,
    bench_alloc,
);

criterion_main!(benches);
