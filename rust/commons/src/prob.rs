use rand::rngs::StdRng;
use rand::Rng;
use rand_distr::{ChiSquared, Distribution, Normal};
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum RDistrib {
    MinMax(f32, f32),
    Normal(f32, f32),
    ChiSquare { k: f32, mult: f32, add: f32 },
    List { values: Vec<f32> },
    WeightedList { values: Vec<(f32, f32)> },
}

#[test]
fn test_serialize_rdistr() {
    let str = serde_json::to_string(&RDistrib::MinMax(0.0, 10.0)).unwrap();
    println!("{}", str);
    let str = serde_json::to_string(&RDistrib::List {
        values: vec![0.0, 1.0, 2.0],
    })
    .unwrap();
    println!("{}", str);
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Weighted<T: Clone> {
    pub prob: f32,
    pub value: T,
}

impl RDistrib {
    pub fn next(&self, rng: &mut StdRng) -> f32 {
        match self {
            RDistrib::MinMax(min, max) => rng.gen_range(*min..*max),

            RDistrib::Normal(mean, std_dev) => {
                let normal = Normal::new(*mean, *std_dev).unwrap();
                normal.sample(rng) as f32
            }

            RDistrib::ChiSquare { k, mult, add } => {
                let d = ChiSquared::new(*k).unwrap();
                (d.sample(rng) * *mult + *add) as f32
            }

            RDistrib::List { values } => {
                let index = rng.gen_range(0..values.len());
                values[index]
            }
            RDistrib::WeightedList { values } => {
                let weights: Vec<f32> = values.iter().map(|(_, value)| *value).collect();
                let index = select_one(rng, &weights).expect("select one receive a empty list");
                values[index].0
            }
        }
    }

    pub fn next_positive(&self, rng: &mut StdRng) -> f32 {
        self.next(rng).max(0.0)
    }

    pub fn next_int(&self, rng: &mut StdRng) -> i32 {
        (self.next(rng).round() as i32).max(0)
    }
}

pub fn select<'a, R: rand::Rng, K: Clone>(
    rng: &mut R,
    candidates: &'a Vec<Weighted<K>>,
) -> Option<&'a K> {
    let weights: Vec<f32> = candidates.iter().map(|i| i.prob).collect();
    select_one(rng, &weights).map(|index| &candidates[index].value)
}

// giving a vector of prob weights, returns the index of the choose sorted option
pub fn select_one<R: rand::Rng>(rng: &mut R, candidates: &Vec<f32>) -> Option<usize> {
    let sum: f32 = candidates.iter().sum();
    let value: f32 = rng.gen();
    let choice: f32 = value * sum;

    candidates
        .iter()
        .enumerate()
        .scan(choice, |state, (index, score)| {
            if *state < 0.0 {
                None
            } else {
                *state -= score;
                Some(index)
            }
        })
        .last()
}

#[cfg(test)]
mod test {
    use super::*;
    use rand::thread_rng;

    #[test]
    fn select_one_should_return_one_of_candidates() {
        let mut rng = thread_rng();
        let list = vec![10.0, 1.0];

        let (total_1, total_2) = (0..50)
            .map(|_| match select_one(&mut rng, &list) {
                Some(0) => (1, 0),
                Some(1) => (0, 1),
                _other => panic!("unexpected result"),
            })
            .fold((0, 0), |(a0, a1), (b0, b1)| (a0 + b0, a1 + b1));

        // check that 1 frequency is grater that "10" times 2 frequency
        assert!(total_1 > total_2 * 5)
    }
}
