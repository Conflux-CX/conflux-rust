// Copyright 2019 Conflux Foundation. All rights reserved.
// Conflux is free software and distributed under GNU General Public License.
// See http://www.gnu.org/licenses/

use cfx_bytes::Bytes;
use cfx_types::U256;
use cfxcore::{
    executive::{Executive, InternalContractMap},
    machine::new_machine_with_builtin,
    state::State,
    statedb::StateDb,
    storage::{state_manager::StateIndex, StorageManagerTrait},
    vm::{Env, Spec},
    vm_factory::VmFactory,
};
use cfxkey::{Generator, KeyPair, Random};
use client::{archive::ArchiveClient, configuration::Configuration};
use criterion::{criterion_group, criterion_main, Benchmark, Criterion};
use parking_lot::{Condvar, Mutex};
use primitives::{Action, Transaction};
use std::{sync::Arc, time::Duration};

fn txexe_benchmark(c: &mut Criterion) {
    let mut conf = Configuration::default();
    conf.raw_conf.mode = Some("test".to_owned());
    let exit = Arc::new((Mutex::new(false), Condvar::new()));
    let handler = ArchiveClient::start(conf, exit).unwrap();
    let kp = KeyPair::from_secret(
        "46b9e861b63d3509c88b7817275a30d22d62c8cd8fa6486ddee35ef0d8e0495f"
            .parse()
            .unwrap(),
    )
    .unwrap();
    let receiver_kp = Random.generate().expect("Fail to generate KeyPair.");

    let tx = Transaction {
        nonce: 0.into(),
        gas_price: U256::from(100u64),
        gas: U256::from(21000u64),
        value: 1.into(),
        action: Action::Call(receiver_kp.address()),
        storage_limit: U256::zero(),
        epoch_height: 0,
        chain_id: 0,
        data: Bytes::new(),
    };
    let tx = tx.sign(kp.secret());
    let machine = new_machine_with_builtin();
    let internal_contract_map = InternalContractMap::new();
    let env = Env {
        number: 0, // TODO: replace 0 with correct cardinal number
        author: Default::default(),
        timestamp: Default::default(),
        difficulty: Default::default(),
        gas_used: U256::zero(),
        gas_limit: tx.gas.clone(),
        last_hashes: Arc::new(vec![]),
    };
    let spec = Spec::new_spec();
    c.bench(
        "Execute 1 transaction",
        Benchmark::new("Execute 1 transaction", move |b| {
            let mut state = State::new(
                StateDb::new(
                    handler
                        .other_components
                        .consensus
                        .data_man
                        .storage_manager
                        .get_state_for_next_epoch(
                            // FIXME: delta height
                            StateIndex::new_for_test_only_delta_mpt(
                                &handler
                                    .other_components
                                    .consensus
                                    .best_block_hash(),
                            ),
                        )
                        .unwrap()
                        .unwrap(),
                ),
                VmFactory::new(1024 * 32),
                0, /* block_number */
            );
            let mut ex = Executive::new(
                &mut state,
                &env,
                &machine,
                &spec,
                &internal_contract_map,
            );
            let mut nonce_increased = false;
            b.iter(|| {
                //ex.transact(&tx, &mut nonce_increased);
                ex.transact(&tx, &mut nonce_increased).unwrap();
                ex.state.clear();
            })
        })
        .measurement_time(Duration::from_secs(10))
        .warm_up_time(Duration::from_secs(10)),
    );
}

criterion_group!(benches, txexe_benchmark);
criterion_main!(benches);
