use criterion::{black_box, criterion_group, criterion_main, Criterion};
use derohe::pow::{astrobwt, salsa20};
use divsufsort::sort_in_place;
use derohe::pow::astrobwt::{sha3, STAGE1_LENGTH};
use criterion_cycles_per_byte::CyclesPerByte;

fn criterion_benchmark(c: &mut Criterion) {
    let input: [u8; 48] = [65, 90, 158, 0, 0, 0, 131, 134, 179, 254, 154, 24, 0, 0, 0, 0, 76, 45, 130, 143, 5, 131, 168, 109, 185, 99, 157, 54, 84, 143, 129, 113, 0, 0, 0, 0, 222, 179, 70, 94, 29, 49, 111, 0, 0, 0, 2, 1];
    let mut key = sha3(&input); // Step 1: calculate SHA3 of input data
    let mut stage1_result = [0u8; STAGE1_LENGTH];

    salsa20::xor_key_stream(
        &mut stage1_result,
        &[0u8; STAGE1_LENGTH],
        &key,
    );
    let mut csa = vec![0; stage1_result.len()];
    let mut sa = vec![0; stage1_result.len()];
    c.bench_function("pow16", |b| b.iter(|| astrobwt::pow16(black_box(&input))));
    c.bench_function("csa", |b| b.iter(|| cdivsufsort::sort_in_place(black_box(&stage1_result), black_box(&mut csa))));
    c.bench_function("sa", |b| b.iter(|| divsufsort::sort_in_place(black_box(&stage1_result), black_box(&mut sa))));
}

fn criterion_benchmark_cycles(c: &mut Criterion<CyclesPerByte>) {
    let input: [u8; 48] = [65, 90, 158, 0, 0, 0, 131, 134, 179, 254, 154, 24, 0, 0, 0, 0, 76, 45, 130, 143, 5, 131, 168, 109, 185, 99, 157, 54, 84, 143, 129, 113, 0, 0, 0, 0, 222, 179, 70, 94, 29, 49, 111, 0, 0, 0, 2, 1];
    let mut key = sha3(&input); // Step 1: calculate SHA3 of input data
    let mut stage1_result = [0u8; STAGE1_LENGTH];

    salsa20::xor_key_stream(
        &mut stage1_result,
        &[0u8; STAGE1_LENGTH],
        &key,
    );
    let mut csa = vec![0; stage1_result.len()];
    let mut sa = vec![0; stage1_result.len()];
    c.bench_function("pow16", |b| b.iter(|| astrobwt::pow16(black_box(&input))));
    c.bench_function("csa", |b| b.iter(|| cdivsufsort::sort_in_place(black_box(&stage1_result), black_box(&mut csa))));
    c.bench_function("sa", |b| b.iter(|| divsufsort::sort_in_place(black_box(&stage1_result), black_box(&mut sa))));
}

criterion_group!(benches1, criterion_benchmark);
criterion_group!(name=benches2; config=Criterion::default().with_measurement(CyclesPerByte);targets= criterion_benchmark_cycles);
// criterion_main!(benches1);
criterion_main!(benches2);