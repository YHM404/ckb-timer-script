# Implement a CKB timer-lock-script

# 1.install dev tools

- **ckb** and **ckb-cli**: [Github](https://github.com/nervosnetwork/ckb/releases) (reference: [https://docs.nervos.org/docs/basics/guides/devchain](https://docs.nervos.org/docs/basics/guides/devchain))
- **[Docker](https://docs.docker.com/get-docker/)**
- **Capsule(latest-version:0.7.4)**: Capsule is a set of tools for Rust developers to develop scripts on CKB which covers the entire lifecycle of script development: writing,debugging,testing and deployment.

# 2.Create a project

```bash
capsule new timer-cell
```

# 3. Coding

Open `timer-cell/contracts/timer-cell/src/entry.rs`, focus on main function

```bash
use crate::error::Error;
pub fn main() -> Result<(), Error> {
    Ok(())
}
```

First, we implement the timer code, because of The script is executed on the RSCV platform, we cannot use timer crates that rely on the std. In order to read the **timestamp**, we must read the header of the block. (reference: [block_structure](https://github.com/nervosnetwork/rfcs/blob/master/rfcs/0027-block-structure/0027-block-structure.md)).

The details are in the code comments.

```bash
let lock_time = load_cell_data(0, Source::Input)?;
let unlock_time = compute_unlock_time(&lock_time)?;
check_if_unlock_time(unlock_time)?;
------------------------------------------------------------------------------------
//lock_time(Unix time) represent how long we want lock this cell 
//after the block is on the chain.
//If We want get the unlock_time, we must get the date-time of the cell
//and plus with lock_time.
pub fn compute_unlock_time(lock_time: &[u8]) -> Result<u64, Error> {
    let lock_time: u64 = u64::from_be_bytes(into_i64_array(lock_time)?);
    let cur_block_time = load_header(0, Source::Input)?.raw().timestamp().unpack();
    let unlock_time = lock_time + cur_block_time;
    Ok(unlock_time)
}
//Traverse the sub-blocks of the cell and get the timestamp of each sub-block.
//if any of headers' timesamp are larger than unlock_time, that means the cell
//can be geted by specific address.
pub fn check_if_unlock_time(unlock_time: u64) -> Result<(), Error> {
    for header in QueryIter::new(load_header, Source::HeaderDep) {
        if unlock_time < header.raw().timestamp().unpack() {
            return Ok(());
        }
    }
    Err(Error::TimeLock)
}
```

Second, we need validate if the address is what we want.

CKB default sign-method algorithm of tx is `[SECP256K1](https://en.bitcoin.it/wiki/Secp256k1)`, so we chose [extern SECP256K1-algorithm crate](https://github.com/jjyr/ckb-dynamic-loading-secp256k1/tree/master/contracts/ckb-dynamic-loading-secp256k1)(implement by **jjyr:** [Github](https://github.com/jjyr)) to decode the signature of tx.

```bash
let script = load_script()?;
let args: Bytes = script.args().unpack();
let pub_key = &args;
let mut context = unsafe { CKBDLContext::<[u8; 128 * 1024]>::new() };
let lib = LibSecp256k1::load(&mut context);
test_validate_blake2b_sighash_all(&lib, &args)?;
```

# 4.Build Script and Test Script

## 4.1 Build Script

CD our project dir, and build it.

```bash
capsule build
```

The script after build in `timer-cell/build/debug`.

## 4.2 Publish our script to CKB-testnet
