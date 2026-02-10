use criterion::{Criterion, criterion_group, criterion_main};
use orx_criterion::{Data, Experiment, Variant};
use rand::prelude::*;
use rand_chacha::ChaCha8Rng;

fn run(c: &mut Criterion) {
    // let data = [
    //     DataSettings(1 << 5),
    //     DataSettings(1 << 10),
    //     DataSettings(1 << 15),
    // ];
    // let variants = [
    //     SearchMethod(StoreType::None),
    //     SearchMethod(StoreType::SortedVec),
    //     SearchMethod(StoreType::HashMap),
    //     SearchMethod(StoreType::BTreeMap),
    // ];

    // TwoSumExp::bench(c, "two_sum", &data, &variants);
}

criterion_group!(benches, run);
criterion_main!(benches);
