#![allow(dead_code)]
use bumpup::{BumpDown, BumpUp};
use criterion::*;
use std::alloc::Layout;
use std::cmp::max;
use std::mem::size_of;

#[derive(Default, Copy, Clone)]
pub struct Small(u8);

#[derive(Default, Copy, Clone)]
pub struct Medium(usize);

#[derive(Default, Copy, Clone)]
#[repr(align(16))]
pub struct Big(usize, usize);

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

pub trait Alloc {
    fn alloc(&mut self, layout: Layout) -> Option<std::ptr::NonNull<u8>>;
    fn clear(&mut self);
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
        self.alloc(layout)
    }
    fn clear(&mut self) {
        self.clear();
    }
}

pub fn alloc<T: Default + From<usize>, A: Alloc>(arena: &mut A, n: usize) {
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

pub fn alloc_slice<T: Copy, A: Alloc>(val: &[T], arena: &mut A, n: usize) {
    let layout = Layout::for_value(val);
    arena.clear();
    for _ in 0..n {
        let ptr = arena.alloc(layout).unwrap();
        unsafe { std::ptr::copy_nonoverlapping(val.as_ptr(), ptr.as_ptr().cast::<T>(), val.len()) }
    }
}

const M: usize = std::mem::align_of::<usize>();
const ALLOCATIONS: usize = 10000;
const LEN: usize = 1;

fn bench_alloc_generic<T: Default + From<usize>>(name: &str, c: &mut Criterion) {
    let mut group = c.benchmark_group(name);
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

fn bench_alloc_slice_generic<T: Default + Copy>(name: &str, c: &mut Criterion) {
    let mut group = c.benchmark_group(name);
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

pub fn bench(c: &mut Criterion) {
    bench_alloc_generic::<Small>("alloc/small", c);
    bench_alloc_generic::<Medium>("alloc/medium", c);
    bench_alloc_generic::<Big>("alloc/big", c);
    bench_alloc_slice_generic::<Small>("alloc_slice/small", c);
    bench_alloc_slice_generic::<Medium>("alloc_slice/medium", c);
    bench_alloc_slice_generic::<Big>("alloc_slice/big", c);
}

criterion_group!(benches, bench);

criterion_main!(benches);
