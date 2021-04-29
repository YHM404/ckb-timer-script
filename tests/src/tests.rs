use super::*;
use ckb_tool::ckb_types::{
    core::PublicKey,
    packed::{CellInputVecBuilder, CellOutputVec, CellOutputVecBuilder},
};
use ckb_tool::rpc_client::RpcClient;
use ckb_tool::{
    ckb_types::{bytes::Bytes, core::TransactionBuilder, packed::Transaction},
    faster_hex::{hex_decode, hex_encode},
};
const MAX_CYCLES: u64 = 10_000_000;
// error numbers
const ERROR_EMPTY_ARGS: i8 = 5;

#[test]
fn test_put_stcrpt_to_testnet() {
    //start a rpc client
    let client = RpcClient::new("http://127.0.0.1:1111");

    //build cell_inputs
    let mut input_cell_tx_hash = [0; 32];
    hex_decode(b"input_cell_tx_hash", &mut input_cell_tx_hash).expect("hex decode");
    let input_cell_tx_hash = H256::from_slice(&input_cell_tx_hash).expect("H256 tx_hash");
    let index = 0;
    let cell_input = build_input_cell(input_cell_tx_hash.0.pack(), index);
    let cell_input_vec = CellInputVecBuilder::default().push(cell_input).build();

    //build cell_outputs
    let mut lock_script_code_hash: [u8; 32] = [0; 32];
    hex_decode(b"lock_script_code_hash", &mut lock_script_code_hash).expect("hex decode");
    let lock_script_code_hash = Byte32::new(lock_script_code_hash);
    let pub_key = PublicKey::from_str("pub key").expect("pub key");
    let cell_output = build_output_cell(1000, pub_key.as_bytes().pack(), lock_script_code_hash);
    let cell_output_vec = CellOutputVecBuilder::default().push(cell_output).build();
    let script_data = Loader::default().load_binary("timer-cell");

    //build tx
    let tx = RawTransaction::new_builder()
        .inputs(cell_input_vec)
        .outputs(cell_output_vec)
        .outputs_data(vec![script_data].pack())
        .build();
    let pri_key = Privkey::from_str("priv key").expect("pub key");
    let tx = sign_tx(tx, pri_key).into();
    let send_tx_hash = client.send_transaction(tx);
}
