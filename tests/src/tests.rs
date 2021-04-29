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

#[test]
fn test_put_timer_stcrpt_to_testnet() {
    let input_cell_tx_hash = b"0x18a103ca921adf533ec2efd6c3312098d1bcd71635e7220086aefb415bc7adf1";
    let index = 0;
    let lock_script_code_hash =
        b"0x9bd7e06f3ecf4be0f2fcd2188b23f1b9fcc88e5d4b65a8637b17723bbda3cce8";
    let pub_key_str = ".....";
    let priv_key_str = ".....";
    let script_data = Loader::default().load_binary("timer-cell");

    //add input_cell dep
    let mut dep_hash = [0; 32];
    hex_decode(
        b"0xf8de3bb47d055cdf460d93a2a6e1b05f7432f9777c8c474abf4eec1d4aee5d37",
        &mut dep_hash,
    );
    let cell_dep = CellDepBuilder::default()
        .out_point(
            OutPointBuilder::default()
                .index((0 as u32).pack())
                .tx_hash(Byte32::new(dep_hash))
                .build(),
        )
        .dep_type(Byte::new(0))
        .build();
    let tx_hash = build_and_sent_tx(
        input_cell_tx_hash,
        index,
        lock_script_code_hash,
        pub_key_str,
        priv_key_str,
        vec![cell_dep].pack(),
        vec![script_data].pack(),
    );
}
