// Import from `core` instead of from `std` since we are in no-std mode
use core::convert::TryInto;
use core::result::Result;
// Import CKB syscalls and structures
// https://nervosnetwork.github.io/ckb-std/riscv64imac-unknown-none-elf/doc/ckb_std/index.html
use crate::error::Error;
use ckb_lib_secp256k1::LibSecp256k1;
use ckb_std::{
    ckb_constants::Source,
    ckb_types::{bytes::Bytes, prelude::*},
    default_alloc,
    dynamic_loading_c_impl::CKBDLContext,
    high_level::{load_cell_data, load_header, load_script, QueryIter},
};

fn test_validate_blake2b_sighash_all(
    lib: &LibSecp256k1,
    expected_pubkey_hash: &[u8],
) -> Result<(), Error> {
    let mut pubkey_hash = [0u8; 20];
    lib.validate_blake2b_sighash_all(&mut pubkey_hash)
        .map_err(|err_code| Error::Secp256k1)?;

    // compare with expected pubkey_hash
    if &pubkey_hash[..] != expected_pubkey_hash {
        return Err(Error::WrongPubkey);
    }
    Ok(())
}

pub fn main() -> Result<(), Error> {
    let script = load_script()?;
    let args: Bytes = script.args().unpack();
    if args.len() != 20 {
        return Err(Error::LengthNotEnough);
    }

    let lock_time = load_cell_data(0, Source::Input)?;
    let unlock_time = compute_unlock_time(&lock_time)?;
    check_if_unlock_time(unlock_time)?;

    // create a DL context with 128K buffer size
    let mut context = unsafe { CKBDLContext::<[u8; 128 * 1024]>::new() };
    let lib = LibSecp256k1::load(&mut context);
    test_validate_blake2b_sighash_all(&lib, &args)?;

    Ok(())
}

pub fn into_i64_array(arr: &[u8]) -> Result<[u8; 8], Error> {
    match arr.try_into() {
        Ok(arr) => Ok(arr),
        Err(_) => Err(Error::Encoding),
    }
}

pub fn compute_unlock_time(lock_time: &[u8]) -> Result<u64, Error> {
    let lock_time: u64 = u64::from_be_bytes(into_i64_array(lock_time)?);
    let cur_block_time = load_header(0, Source::Input)?.raw().timestamp().unpack();
    let unlock_time = lock_time + cur_block_time;
    Ok(unlock_time)
}

pub fn check_if_unlock_time(unlock_time: u64) -> Result<(), Error> {
    for header in QueryIter::new(load_header, Source::HeaderDep) {
        if unlock_time < header.raw().timestamp().unpack() {
            return Ok(());
        }
    }
    Err(Error::TimeLock)
}
