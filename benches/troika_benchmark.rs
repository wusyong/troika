#[macro_use]
extern crate criterion;

use criterion::Criterion;
use troika_rust::troika::Troika;
use troika_rust::ftroika::Ftroika;

fn basic_troika() {
    let mut troika = Troika::default();
    let input = [0u8; 242];
    let mut output = [0u8; 243];

    troika.absorb(&input);
    troika.squeeze(&mut output);
}

fn basic_ftroika() {
    let mut ftroika = Ftroika::default();
    let input = [0u8; 8019];
    let mut output = [0u8; 243];

    ftroika.absorb(&input);
    ftroika.finalize();
    ftroika.squeeze(&mut output);
}

fn criterion_benchmark(c: &mut Criterion) {
    c.bench_function("Ftroika with input of 8019 zeros", |b| {
        b.iter(|| basic_ftroika())
    });
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
