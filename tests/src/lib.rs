use ckb_tool::ckb_types::{
    core::TransactionView,
    packed::{
        self, Byte, Byte32, BytesOpt, BytesOptBuilder, CellInput, CellInputBuilder, CellOutput,
        CellOutputBuilder, OutPoint, OutPointBuilder, RawTransaction, ScriptBuilder, Transaction,
        WitnessArgs,
    },
    prelude::{Builder, Entity, Pack, PackVec},
    H256,
};
use ckb_tool::{ckb_crypto::secp::Privkey, ckb_hash::new_blake2b, ckb_types::bytes::Bytes};
use std::fs;
use std::path::PathBuf;
use std::str::FromStr;
use std::{env, u64};

#[cfg(test)]
mod tests;

const TEST_ENV_VAR: &str = "CAPSULE_TEST_ENV";

//only for one-cell-input and one-cell-output tx with no type-script.
pub fn sign_tx(tx: RawTransaction, pri_key: Privkey) -> Transaction {
    let mut hasher = new_blake2b();
    //hash the tx hash
    hasher.update(&tx.calc_tx_hash().raw_data());
    //witnessArgs
    let witness_args = WitnessArgs::default();
    let mut dummy_lock = Vec::new();
    dummy_lock.resize(65, 0);
    let dummy_lock: Bytes = dummy_lock.into();
    let dummy_lock: BytesOpt = BytesOptBuilder::default()
        .set(Some(dummy_lock.pack()))
        .build();
    let witness_args_bytes = witness_args
        .clone()
        .as_builder()
        .lock(dummy_lock)
        .build()
        .as_bytes();
    let witness_args_len = witness_args_bytes.len() as u64;
    hasher.update(&witness_args_len.to_le_bytes());
    hasher.update(&witness_args_bytes);
    let mut sig_hash = [0; 32];
    hasher.finalize(&mut sig_hash);
    //sign tx
    let signature = pri_key
        .sign_recoverable(&H256::from(sig_hash))
        .expect("sign tx");
    let signature_bytes: Bytes = signature.serialize().into();
    let signed_witness = witness_args
        .as_builder()
        .lock(Some(signature_bytes).pack())
        .build()
        .as_bytes()
        .pack();
    //put the signature back to the first witness
    let witnesses_with_lock = vec![signed_witness];
    Transaction::from_slice(tx.as_slice())
        .expect("raw_tx")
        .as_builder()
        .witnesses(witnesses_with_lock.pack())
        .build()
}

pub fn build_input_cell(tx_hash: Byte32, index: u32) -> CellInput {
    let out_point = OutPointBuilder::default()
        .tx_hash(tx_hash)
        .index(index.pack())
        .build();
    CellInputBuilder::default()
        .previous_output(out_point)
        .build()
}

pub fn build_output_cell(
    capacity: u64,
    args: packed::Bytes,
    code_hash: packed::Byte32,
) -> CellOutput {
    let script = ScriptBuilder::default()
        .args(args)
        .code_hash(code_hash)
        .hash_type(Byte::new(1))
        .build();
    CellOutputBuilder::default()
        .capacity(capacity.pack())
        .lock(script)
        .build()
}
pub fn hex_to_bytes(s: &str) -> Option<Vec<u8>> {
    if s.len() % 2 == 0 {
        (0..s.len())
            .step_by(2)
            .map(|i| {
                s.get(i..i + 2)
                    .and_then(|sub| u8::from_str_radix(sub, 16).ok())
            })
            .collect()
    } else {
        None
    }
}

pub enum TestEnv {
    Debug,
    Release,
}

impl FromStr for TestEnv {
    type Err = &'static str;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "debug" => Ok(TestEnv::Debug),
            "release" => Ok(TestEnv::Release),
            _ => Err("no match"),
        }
    }
}

pub struct Loader(PathBuf);

impl Default for Loader {
    fn default() -> Self {
        let test_env = match env::var(TEST_ENV_VAR) {
            Ok(val) => val.parse().expect("test env"),
            Err(_) => TestEnv::Debug,
        };
        Self::with_test_env(test_env)
    }
}

impl Loader {
    fn with_test_env(env: TestEnv) -> Self {
        let load_prefix = match env {
            TestEnv::Debug => "debug",
            TestEnv::Release => "release",
        };
        let dir = env::current_dir().unwrap();
        let mut base_path = PathBuf::new();
        base_path.push(dir);
        base_path.push("..");
        base_path.push("build");
        base_path.push(load_prefix);
        Loader(base_path)
    }

    pub fn load_binary(&self, name: &str) -> Bytes {
        let mut path = self.0.clone();
        path.push(name);
        fs::read(path).expect("binary").into()
    }
}
