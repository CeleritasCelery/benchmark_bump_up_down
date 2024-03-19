#![allow(dead_code)]
use bumpup::{BumpDown, BumpUp};
use criterion::*;
use std::alloc::Layout;
use std::cmp::max;
use std::mem::size_of;

#[derive(Default, Hash, Copy, Clone)]
struct Small(u8);

#[derive(Default, Hash, Copy, Clone)]
struct Medium(usize);

#[derive(Default, Hash, Copy, Clone)]
#[repr(align(16))]
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

impl<const M: usize> Alloc for BumpUp<M> {
    fn alloc(&mut self, layout: Layout) -> Option<std::ptr::NonNull<u8>> {
        self.alloc(layout)
    }
    fn clear(&mut self) {
        self.clear();
    }
}

impl<const M: usize> Alloc for BumpDown<M> {
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
    for _ in 0..n {
        let ptr = arena.alloc(layout).unwrap();
        unsafe {
            ptr.as_ptr()
                .cast::<T>()
                .write(T::from(ptr.as_ptr() as usize))
        }
    }
}

fn alloc_slice<T: Copy, A: Alloc>(val: &[T], arena: &mut A, n: usize) {
    let layout = Layout::for_value(val);
    arena.clear();
    for _ in 0..n {
        let ptr = arena.alloc(layout).unwrap();
        unsafe { std::ptr::copy_nonoverlapping(val.as_ptr(), ptr.as_ptr().cast::<T>(), val.len()) }
    }
}

const M: usize = std::mem::align_of::<usize>();
const ALLOCATIONS: usize = 10000;

fn bench_alloc(c: &mut Criterion) {
    {
        let mut group = c.benchmark_group("alloc/medium");
        type T = Medium;
        let capacity = ALLOCATIONS * max(size_of::<T>(), M);
        let mut arena = black_box(BumpUp::with_capacity(capacity));
        group.bench_function("up", |b| {
            b.iter(|| alloc::<T, BumpUp<M>>(&mut arena, ALLOCATIONS))
        });
        let mut arena = black_box(BumpDown::with_capacity(capacity));
        group.bench_function("down", |b| {
            b.iter(|| alloc::<T, BumpDown<M>>(&mut arena, ALLOCATIONS))
        });
    }
    {
        let mut group = c.benchmark_group("alloc/small");
        type T = Small;
        let capacity = ALLOCATIONS * max(size_of::<T>(), M);
        let mut arena = black_box(BumpUp::with_capacity(capacity));
        group.bench_function("up", |b| {
            b.iter(|| alloc::<T, BumpUp<M>>(&mut arena, ALLOCATIONS))
        });
        let mut arena = black_box(BumpDown::with_capacity(capacity));
        group.bench_function("down", |b| {
            b.iter(|| alloc::<T, BumpDown<M>>(&mut arena, ALLOCATIONS))
        });
    }
    {
        let mut group = c.benchmark_group("alloc/big");
        type T = Big;
        let capacity = ALLOCATIONS * max(size_of::<T>(), M);
        let mut arena = black_box(BumpUp::with_capacity(capacity));
        group.bench_function("up", |b| {
            b.iter(|| alloc::<T, BumpUp<M>>(&mut arena, ALLOCATIONS))
        });
        let mut arena = black_box(BumpDown::with_capacity(capacity));
        group.bench_function("down", |b| {
            b.iter(|| alloc::<T, BumpDown<M>>(&mut arena, ALLOCATIONS))
        });
    }
}

fn bench_alloc_slice(c: &mut Criterion) {
    const LEN: usize = 1;
    {
        let mut group = c.benchmark_group("alloc_slice/small");
        type T = Small;
        let capacity = ALLOCATIONS * LEN * max(size_of::<T>(), M);
        let val: Box<[T]> = black_box(Box::new([T::default(); LEN]));
        let mut arena = black_box(BumpUp::with_capacity(capacity));
        group.bench_function("up", |b| {
            b.iter(|| alloc_slice::<T, BumpUp<M>>(&val, &mut arena, ALLOCATIONS))
        });
        let mut arena = black_box(BumpDown::with_capacity(capacity));
        group.bench_function("down", |b| {
            b.iter(|| alloc_slice::<T, BumpDown<M>>(&val, &mut arena, ALLOCATIONS))
        });
    }
    {
        let mut group = c.benchmark_group("alloc_slice/medium");
        type T = Medium;
        let capacity = ALLOCATIONS * LEN * max(size_of::<T>(), M);
        let val: Box<[T]> = black_box(Box::new([T::default(); LEN]));
        let mut arena = black_box(BumpUp::with_capacity(capacity));
        group.bench_function("up", |b| {
            b.iter(|| alloc_slice::<T, BumpUp<M>>(&val, &mut arena, ALLOCATIONS))
        });
        let mut arena = black_box(BumpDown::with_capacity(capacity));
        group.bench_function("down", |b| {
            b.iter(|| alloc_slice::<T, BumpDown<M>>(&val, &mut arena, ALLOCATIONS))
        });
    }
    {
        let mut group = c.benchmark_group("alloc_slice/big");
        type T = Big;
        let capacity = ALLOCATIONS * LEN * max(size_of::<T>(), M);
        let val: Box<[T]> = black_box(Box::new([T::default(); LEN]));
        let mut arena = black_box(BumpUp::with_capacity(capacity));
        group.bench_function("up", |b| {
            b.iter(|| alloc_slice::<T, BumpUp<M>>(&val, &mut arena, ALLOCATIONS))
        });
        let mut arena = black_box(BumpDown::with_capacity(capacity));
        group.bench_function("down", |b| {
            b.iter(|| alloc_slice::<T, BumpDown<M>>(&val, &mut arena, ALLOCATIONS))
        });
    }
}

criterion_group!(benches, bench_alloc, bench_alloc_slice,);

criterion_main!(benches);
