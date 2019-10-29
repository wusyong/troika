use troika::ftroika::*;
use troika::troika::*;
use rand::{thread_rng, Rng};

#[test]
fn random_hash() {
    let mut ftroika = Ftroika::default();
    let mut troika = Troika::default();
    let mut foutput = [0u8; 243];
    let mut output = [0u8; 243];
    let mut input = [0u8; 243];
    let mut rng = thread_rng();

    for _ in 0..10 {
        for trit in input.iter_mut() {
            *trit = rng.gen_range(0, 3);
        }

        ftroika.absorb(&input);
        ftroika.squeeze(&mut foutput);
        ftroika.reset();

        troika.absorb(&input);
        troika.squeeze(&mut output);
        troika.reset();

        assert!(
            foutput.iter().zip(output.iter()).all(|(a, b)| a == b),
            "Arrays are not equal"
        );
    }
}
