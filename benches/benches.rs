use bump_up_or_down::{BumpDown, BumpUp};
use criterion::*;
use std::alloc::Layout;
use std::cmp::max;
use std::mem::size_of;

pub trait Alloc {
    fn alloc(&mut self, layout: Layout) -> Option<std::ptr::NonNull<u8>>;
    fn clear(&mut self);
}

struct BumpUpOrig(BumpUp<1>);
impl Alloc for BumpUpOrig {
    fn alloc(&mut self, layout: Layout) -> Option<std::ptr::NonNull<u8>> {
        self.0.alloc_orig(layout)
    }

    fn clear(&mut self) {
        self.0.clear()
    }
}

struct BumpDownOrig(BumpDown<1>);
impl Alloc for BumpDownOrig {
    fn alloc(&mut self, layout: Layout) -> Option<std::ptr::NonNull<u8>> {
        self.0.alloc_orig(layout)
    }

    fn clear(&mut self) {
        self.0.clear()
    }
}

struct BumpUpByte(BumpUp<1>);
impl Alloc for BumpUpByte {
    fn alloc(&mut self, layout: Layout) -> Option<std::ptr::NonNull<u8>> {
        self.0.alloc(layout)
    }

    fn clear(&mut self) {
        self.0.clear()
    }
}

struct BumpUpWord(BumpUp<8>);
impl Alloc for BumpUpWord {
    fn alloc(&mut self, layout: Layout) -> Option<std::ptr::NonNull<u8>> {
        self.0.alloc(layout)
    }

    fn clear(&mut self) {
        self.0.clear()
    }
}

struct BumpUp2Word(BumpUp<16>);
impl Alloc for BumpUp2Word {
    fn alloc(&mut self, layout: Layout) -> Option<std::ptr::NonNull<u8>> {
        self.0.alloc(layout)
    }

    fn clear(&mut self) {
        self.0.clear()
    }
}

struct BumpDownWord(BumpDown<8>);
impl Alloc for BumpDownWord {
    fn alloc(&mut self, layout: Layout) -> Option<std::ptr::NonNull<u8>> {
        self.0.alloc(layout)
    }

    fn clear(&mut self) {
        self.0.clear()
    }
}

struct BumpDown2Word(BumpDown<16>);
impl Alloc for BumpDown2Word {
    fn alloc(&mut self, layout: Layout) -> Option<std::ptr::NonNull<u8>> {
        self.0.alloc(layout)
    }

    fn clear(&mut self) {
        self.0.clear()
    }
}

pub fn alloc<T, A: Alloc>(arena: &mut A, n: usize) {
    let layout = Layout::new::<T>();
    arena.clear();
    for _ in 0..n {
        let ptr = arena.alloc(layout).unwrap();
        black_box(ptr);
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

const MIN_ALIGN: usize = std::mem::align_of::<u128>();
const ALLOCATIONS: usize = 10000;
const LEN: usize = 1;

fn bench_alloc_generic<T: Default>(c: &mut Criterion) {
    let mut group = c.benchmark_group(format!("alloc/{}", std::any::type_name::<T>()));
    let capacity = ALLOCATIONS * max(size_of::<T>(), MIN_ALIGN);
    let mut arena = black_box(BumpUpOrig(BumpUp::with_capacity(capacity)));
    group.bench_function("orig/up", |b| {
        b.iter(|| alloc::<T, _>(&mut arena, ALLOCATIONS))
    });
    let mut arena = black_box(BumpDownOrig(BumpDown::with_capacity(capacity)));
    group.bench_function("orig/down", |b| {
        b.iter(|| alloc::<T, _>(&mut arena, ALLOCATIONS))
    });
    let mut arena = black_box(BumpUpByte(BumpUp::with_capacity(capacity)));
    group.bench_function("align1/up", |b| {
        b.iter(|| alloc::<T, _>(&mut arena, ALLOCATIONS))
    });
    let mut arena = black_box(BumpUpWord(BumpUp::with_capacity(capacity)));
    group.bench_function("align8/up", |b| {
        b.iter(|| alloc::<T, _>(&mut arena, ALLOCATIONS))
    });
    let mut arena = black_box(BumpDownWord(BumpDown::with_capacity(capacity)));
    group.bench_function("align8/down", |b| {
        b.iter(|| alloc::<T, _>(&mut arena, ALLOCATIONS))
    });
    let mut arena = black_box(BumpUp2Word(BumpUp::with_capacity(capacity)));
    group.bench_function("align16/up", |b| {
        b.iter(|| alloc::<T, _>(&mut arena, ALLOCATIONS))
    });
    let mut arena = black_box(BumpDown2Word(BumpDown::with_capacity(capacity)));
    group.bench_function("align16/down", |b| {
        b.iter(|| alloc::<T, _>(&mut arena, ALLOCATIONS))
    });
}

fn bench_alloc_slice_generic<T: Default + Copy>(c: &mut Criterion) {
    let mut group = c.benchmark_group(format!("alloc_slice/{}", std::any::type_name::<T>()));
    let capacity = ALLOCATIONS * LEN * max(size_of::<T>(), MIN_ALIGN);
    let val: Box<[T]> = black_box(Box::new([T::default(); LEN]));

    let mut arena = black_box(BumpUpOrig(BumpUp::with_capacity(capacity)));
    group.bench_function("orig/up", |b| {
        b.iter(|| alloc_slice::<T, _>(&val, &mut arena, ALLOCATIONS))
    });
    let mut arena = black_box(BumpDownOrig(BumpDown::with_capacity(capacity)));
    group.bench_function("orig/down", |b| {
        b.iter(|| alloc_slice::<T, _>(&val, &mut arena, ALLOCATIONS))
    });
    let mut arena = black_box(BumpUpByte(BumpUp::with_capacity(capacity)));
    group.bench_function("align1/up", |b| {
        b.iter(|| alloc_slice::<T, _>(&val, &mut arena, ALLOCATIONS))
    });
    let mut arena = black_box(BumpUpWord(BumpUp::with_capacity(capacity)));
    group.bench_function("align8/up", |b| {
        b.iter(|| alloc_slice::<T, _>(&val, &mut arena, ALLOCATIONS))
    });
    let mut arena = black_box(BumpDownWord(BumpDown::with_capacity(capacity)));
    group.bench_function("align8/down", |b| {
        b.iter(|| alloc_slice::<T, _>(&val, &mut arena, ALLOCATIONS))
    });
}

pub fn bench(c: &mut Criterion) {
    bench_alloc_generic::<u8>(c);
    bench_alloc_generic::<u64>(c);
    bench_alloc_generic::<u128>(c);
    bench_alloc_slice_generic::<u8>(c);
    bench_alloc_slice_generic::<u64>(c);
    bench_alloc_slice_generic::<u128>(c);
}

criterion_group!(benches, bench);

criterion_main!(benches);
