use ckb_sdk::{
    constants::SIGHASH_TYPE_HASH,
    rpc::CkbRpcClient,
    traits::{
        DefaultCellCollector, DefaultCellDepResolver, DefaultHeaderDepResolver,
        DefaultTransactionDependencyProvider, SecpCkbRawKeySigner,
    },
    tx_builder::{transfer::CapacityTransferBuilder, CapacityBalancer, TxBuilder},
    unlock::{ScriptUnlocker, SecpSighashUnlocker},
    Address, HumanCapacity, ScriptId,
};
use ckb_types::{
    bytes::Bytes,
    core::BlockView,
    h256,
    packed::{CellOutput, Script, WitnessArgs},
    prelude::*,
};
use std::{collections::HashMap, str::FromStr};

fn main() {
    // Get an address to use
    // We decode this from underlying lock script
    let add_str = "ckb1qzda0cr08m85hc8jlnfp3zer7xulejywt49kt2rr0vthywaa50xwsqgvf0k9sc40s3azmpfvhyuudhahpsj72tsr8cx3d";
    let addr = Address::from_str(add_str).unwrap();
    let network = addr.network();
    // extract actual lock script(code_hash, args, hash_type)
    let script: Script = addr.payload().into();
    println!("Decoded address into lock script: {:?}", script);
    // Build and send a tx
    let ckb_rpc = "https://testnet.ckb.dev:8114";
    let sender = Address::from_str("ckt1qzda0cr08m85hc8jlnfp3zer7xulejywt49kt2rr0vthywaa50xwsqf7v2xsyj0p8szesqrwqapvvygpc8hzg9sku954v").unwrap();
    let sender_key = secp256k1::SecretKey::from_slice(
        h256!("0xef4dfe655b3df20838bdd16e20afc70dfc1b9c3e87c54c276820315a570e6555").as_bytes(),
    )
    .unwrap();
    let receiver = Address::from_str(
            "ckt1qzda0cr08m85hc8jlnfp3zer7xulejywt49kt2rr0vthywaa50xwsqvglkprurm00l7hrs3rfqmmzyy3ll7djdsujdm6z",
        )
        .unwrap();
    let capacity = HumanCapacity::from_str("100.0").unwrap();
    // Build the unlocker --- this is how we sign the tx
    let signer = SecpCkbRawKeySigner::new_with_secret_keys(vec![sender_key]);
    let sighash_unlocker = SecpSighashUnlocker::from(Box::new(signer) as Box<_>);
    let sighash_script_id = ScriptId::new_type(SIGHASH_TYPE_HASH.clone());
    let mut unlockers = HashMap::default();
    unlockers.insert(
        sighash_script_id,
        Box::new(sighash_unlocker) as Box<dyn ScriptUnlocker>,
    );
    let placeholder_witness = WitnessArgs::new_builder()
        .lock(Some(Bytes::from(vec![0u8; 65])).pack())
        .build();
    let balancer = CapacityBalancer::new_simple(sender.payload().into(), placeholder_witness, 1000);
    // Connect to ckb node and set up resolvers
    let mut ckb_client = CkbRpcClient::new(ckb_rpc);
    let cell_dep_resolver = {
        let genesis_block = ckb_client.get_block_by_number(0.into()).unwrap().unwrap();
        DefaultCellDepResolver::from_genesis(&BlockView::from(genesis_block)).unwrap()
    };
    let header_dep_resolver = DefaultHeaderDepResolver::new(ckb_rpc);
    let mut cell_collector = DefaultCellCollector::new(ckb_rpc);
    let tx_dep_provider = DefaultTransactionDependencyProvider::new(ckb_rpc, 10);

    // Build tx
    let output = CellOutput::new_builder()
        .lock(Script::from(&receiver))
        .capacity(capacity.0.pack())
        .build();
    let builder = CapacityTransferBuilder::new(vec![(output, Bytes::default())]);
    let (tx, _) = builder
        .build_unlocked(
            &mut cell_collector,
            &cell_dep_resolver,
            &header_dep_resolver,
            &tx_dep_provider,
            &balancer,
            &unlockers,
        )
        .unwrap();

    println!(
        "Transaction built successfully: {} inputs, {} outputs",
        tx.inputs().len(),
        tx.outputs().len(),
    );
}
