use prio::field::{FieldElement, FieldElementWithInteger};
use rand::prelude::*;

pub trait Noise<F: FieldElement> {
    fn sample_from_bitvec(&self, bitvec: &[F], num_shares: usize) -> Vec<F>;
    fn bitvec_len(&self) -> usize;
}

/// As defined in ia.cr/2022/1391.
pub struct NBin<F> {
    n: usize,
    n_div_2: F,
    m_inv: F,
}

impl<F: FieldElementWithInteger> NBin<F> {
    pub fn new(n: usize, m: usize) -> Self {
        Self {
            n,
            n_div_2: F::from(F::Integer::try_from(n).unwrap())
                / F::from(F::Integer::try_from(2).unwrap()),
            m_inv: F::from(F::Integer::try_from(m).unwrap()).inv(),
        }
    }
}

impl<F: FieldElementWithInteger> Noise<F> for NBin<F> {
    fn sample_from_bitvec(&self, bitvec: &[F], num_shares: usize) -> Vec<F> {
        let num_shares_inv = F::from(F::Integer::try_from(num_shares).unwrap()).inv();
        let y = bitvec
            .iter()
            .cloned()
            .reduce(|sum, summand| sum + summand)
            .unwrap_or(F::zero());
        vec![self.m_inv * (y - self.n_div_2 * num_shares_inv)]
    }

    fn bitvec_len(&self) -> usize {
        self.n
    }
}

pub mod testing {
    use super::*;

    use prio::field::{random_vector, FieldElementWithInteger};

    pub fn split_vec<F: FieldElement, const SHARES: usize>(inp: &[F]) -> [Vec<F>; SHARES] {
        let mut outp = Vec::with_capacity(SHARES);
        outp.push(inp.to_vec());

        for _ in 1..SHARES {
            let share: Vec<F> =
                random_vector(inp.len()).expect("failed to generate a random vector");
            for (x, y) in outp[0].iter_mut().zip(&share) {
                *x -= *y;
            }
            outp.push(share);
        }

        outp.try_into().unwrap()
    }

    pub fn random_bitvec<F: FieldElementWithInteger>(len: usize) -> Vec<F> {
        let mut rng = thread_rng();
        let mut bitvec = Vec::with_capacity(len);
        for _ in 0..len {
            bitvec.push(F::from(
                F::Integer::try_from(rng.gen::<bool>() as usize).unwrap(),
            ));
        }
        bitvec
    }
}

#[cfg(test)]
mod tests {
    use prio::field::Field128;

    use super::*;

    #[test]
    fn nbin_linearity() {
        const TRIALS: usize = 100;
        const SHARES: usize = 5;
        let dist = NBin::new(TRIALS, 1);
        let bitvec = testing::random_bitvec::<Field128>(dist.bitvec_len());
        let want = dist.sample_from_bitvec(&bitvec, 1);
        let got = testing::split_vec::<_, SHARES>(&bitvec)
            .iter()
            .map(|bitvec_share| dist.sample_from_bitvec(bitvec_share, SHARES))
            .reduce(|mut sample, sample_share| {
                for (x, y) in sample.iter_mut().zip(sample_share.into_iter()) {
                    *x += y;
                }
                sample
            })
            .unwrap();
        assert_eq!(got, want);
    }
}
