#[macro_use]
extern crate criterion;

use criterion::Criterion;
use troika_rust::stroika::Stroika;

fn basic_stroika() {
    let mut stroika = Stroika::default();
    let input = [0u8; 243];
    let mut output = [0u8; 243];

    stroika.absorb(&input);
    stroika.squeeze(&mut output);
}

fn criterion_benchmark(c: &mut Criterion) {
    c.bench_function("Troika with input of 243 zeros", |b| b.iter(|| basic_stroika()));
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);