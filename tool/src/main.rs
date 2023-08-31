use plotpy::{Histogram, Plot};
use prio::field::{Field128, FieldElementWithInteger};
use vdaf_stuff::{testing, NBin, Noise};

fn decode(elem: Field128) -> i128 {
    let uint = u128::from(elem);
    if uint < Field128::modulus() / 2 {
        i128::try_from(uint).unwrap()
    } else {
        -i128::try_from(Field128::modulus() - uint).unwrap()
    }
}

fn main() {
    let dist = NBin::new(100, 1);
    let samples = std::iter::repeat_with(|| {
        let bitvec = testing::random_bitvec::<Field128>(dist.bitvec_len());
        let sample = dist
            .sample_from_bitvec(&bitvec, 1)
            .into_iter()
            .map(decode)
            .collect::<Vec<_>>();
        sample[0]
    })
    .take(1_000)
    .collect::<Vec<_>>();
    println!("{samples:?}");

    let mut histogram = Histogram::new();
    histogram.draw(&vec![samples], &["values"]);

    let mut plot = Plot::new();
    plot.add(&histogram);
    plot.save_and_show("/tmp/dbin_samples.svg").unwrap();
}
