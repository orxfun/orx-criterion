use criterion::{Criterion, criterion_group, criterion_main};
use orx_criterion::{AlgFactors, Data, Experiment};
use rand::prelude::*;
use rand_chacha::ChaCha8Rng;
use std::collections::{BTreeMap, HashMap};

// https://leetcode.com/problems/two-sum/description/

// data

struct DataSettings(usize);

impl Data for DataSettings {
    fn factor_names() -> Vec<&'static str> {
        vec!["len"]
    }

    fn factor_values(&self) -> Vec<String> {
        vec![self.0.to_string()]
    }
}

// variants

#[derive(Debug)]
enum StoreType {
    None,
    SortedVec,
    HashMap,
    BTreeMap,
}

struct SearchMethod(StoreType);

impl AlgFactors for SearchMethod {
    fn factor_names() -> Vec<&'static str> {
        vec!["store-type"]
    }

    fn factor_values(&self) -> Vec<String> {
        vec![format!("{:?}", self.0)]
    }
}

trait IndexOf<'a> {
    fn from_array(array: &'a [i64]) -> Self;

    fn index_of(&self, complement: i64) -> Option<usize>;
}

impl<'a> IndexOf<'a> for &'a [i64] {
    fn from_array(array: &'a [i64]) -> Self {
        array
    }

    fn index_of(&self, complement: i64) -> Option<usize> {
        self.iter().position(|x| *x == complement)
    }
}

struct SortedVec(Vec<(i64, usize)>);
impl IndexOf<'_> for SortedVec {
    fn from_array(array: &'_ [i64]) -> Self {
        let mut vec: Vec<_> = array
            .iter()
            .enumerate()
            .map(|(idx, val)| (*val, idx))
            .collect();
        vec.sort();
        SortedVec(vec)
    }

    fn index_of(&self, complement: i64) -> Option<usize> {
        match self.0.binary_search_by_key(&complement, |(val, _)| *val) {
            Ok(idx) => Some(self.0[idx].1),
            Err(_) => None,
        }
    }
}

impl IndexOf<'_> for HashMap<i64, usize> {
    fn from_array(array: &'_ [i64]) -> Self {
        array
            .iter()
            .enumerate()
            .map(|(idx, val)| (*val, idx))
            .collect()
    }

    fn index_of(&self, complement: i64) -> Option<usize> {
        self.get(&complement).copied()
    }
}

impl IndexOf<'_> for BTreeMap<i64, usize> {
    fn from_array(array: &'_ [i64]) -> Self {
        array
            .iter()
            .enumerate()
            .map(|(idx, val)| (*val, idx))
            .collect()
    }

    fn index_of(&self, complement: i64) -> Option<usize> {
        self.get(&complement).copied()
    }
}

// generic algorithm

fn algorithm<'a, S: IndexOf<'a>>(array: &'a [i64], target: i64) -> Option<[usize; 2]> {
    let store = S::from_array(array);
    for (i, a) in array.iter().enumerate() {
        let b = target - a;
        if let Some(j) = store.index_of(b) {
            return Some([i, j]);
        }
    }
    None
}

// experiment

struct Input {
    array: Vec<i64>,
    indices: Option<[usize; 2]>,
}

struct TwoSumExp;

impl Experiment for TwoSumExp {
    type Data = DataSettings;

    type Variant = SearchMethod;

    type Input = Input;

    type Output = Option<[usize; 2]>;

    fn input(data: &Self::Data) -> Self::Input {
        let mut rng = ChaCha8Rng::seed_from_u64(42);
        let n = data.0;
        let mut array: Vec<_> = (0..data.0).map(|_| rng.random_range(3..n as i64)).collect();
        let i = array[n / 2] as usize;
        let j = array[3 * n / 4] as usize;
        array[i] = 1;
        array[j] = 2;
        let indices = Some([i, j]);
        Input { array, indices }
    }

    fn execute(variant: &Self::Variant, input: &Self::Input) -> Self::Output {
        let array = &input.array;
        match variant.0 {
            StoreType::None => algorithm::<&[i64]>(array, 3),
            StoreType::SortedVec => algorithm::<SortedVec>(array, 3),
            StoreType::HashMap => algorithm::<HashMap<_, _>>(array, 3),
            StoreType::BTreeMap => algorithm::<BTreeMap<_, _>>(array, 3),
        }
    }

    fn validate_output(_: &Self::Data, input: &Self::Input, output: &Self::Output) {
        assert_eq!(input.indices, *output);
        assert_eq!(
            output.map(|[i, j]| input.array[i] + input.array[j]),
            Some(3)
        );
    }
}

fn run(c: &mut Criterion) {
    let data = [
        DataSettings(1 << 5),
        DataSettings(1 << 10),
        DataSettings(1 << 15),
    ];
    let variants = [
        SearchMethod(StoreType::None),
        SearchMethod(StoreType::SortedVec),
        SearchMethod(StoreType::HashMap),
        SearchMethod(StoreType::BTreeMap),
    ];

    TwoSumExp::bench(c, "two_sum", &data, &variants);
}

criterion_group!(benches, run);
criterion_main!(benches);
