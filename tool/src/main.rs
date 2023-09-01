use plotpy::{Histogram, Plot};
use prio::field::{Field128, FieldElement, FieldElementWithInteger};
use rand::prelude::*;
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
    const SHARES: usize = 2;
    let shares_inv = Field128::from(SHARES as u128).inv();
    let mut rng = thread_rng();

    let dist = NBin::new(10, 1);
    let samples = std::iter::repeat_with(|| {
        // Client provides secret shares of a bitvector.
        let bitvec_shares =
            testing::split_vec::<Field128, SHARES>(&testing::random_bitvec(dist.bitvec_len()));

        // Hedge against malicious Clients.
        let mut invertvec = Vec::with_capacity(dist.bitvec_len());
        for _ in 0..dist.bitvec_len() {
            invertvec.push(rng.gen::<bool>());
        }

        // Each aggregator computes its sample of the noise.
        let sample_shares = bitvec_shares.into_iter().map(|mut bitvec_share| {
            for (bit_share, invert) in bitvec_share.iter_mut().zip(invertvec.iter()) {
                if *invert {
                    *bit_share = shares_inv - *bit_share;
                }
            }
            dist.sample_from_bitvec(&bitvec_share, SHARES)
        });

        // When used with a VDAF, each Aggregator would add its sample share into the its aggregate
        // result. Here we just want to know what the distribution of the noise looks like, so
        // we'll unshard it.
        let sample = sample_shares
            .reduce(|mut sample, sample_share| {
                for (x, y) in sample.iter_mut().zip(sample_share.into_iter()) {
                    *x += y;
                }
                sample
            })
            .unwrap();

        decode(sample[0])
    })
    .take(1_000)
    .collect::<Vec<_>>();
    println!("{samples:?}");

    let mut histogram = Histogram::new();
    histogram.draw(&vec![samples], &["values"]);

    let mut plot = Plot::new();
    plot.add(&histogram);
    plot.save("/tmp/dbin_samples.png").unwrap();
}
