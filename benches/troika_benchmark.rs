#[macro_use]
extern crate criterion;

use criterion::Criterion;
use troika_rust::troika::Troika;

fn basic_troika() {
    let mut troika = Troika::default();
    let input = [0u8; 242];
    let mut output = [0u8; 243];

    troika.absorb(&input);
    troika.squeeze(&mut output);
}

fn criterion_benchmark(c: &mut Criterion) {
    c.bench_function("Troika with input of 243 zeros", |b| {
        b.iter(|| basic_troika())
    });
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
