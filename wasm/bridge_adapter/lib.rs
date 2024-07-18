#![no_std]
#![no_main]

use we_cdk::*;

const SEP: String = "__";
const KEY_INIT: String = "INIT";
const KEY_THIS: String = "THIS";
const KEY_MULTISIG: String = "MULTISIG";
const KEY_STATUS: String = "STATUS";
const KEY_PAUSED: String = "PAUSED";
const KEY_PAUSER: String = "PAUSER";
const KEY_ALLOWANCE: String = "ALLOWANCE";
const KEY_ADAPTER: String = "ADAPTER";

fn validate_address(address: &[u8]) -> bool {
    // 86 for mainnet, 84 for testnet
    address.len() == 26 && (address.starts_with(&[1, 86]) || address.starts_with(&[1, 84]))
}

fn validate_contract(contract: &[u8]) -> bool {
    contract.len() == 32
}

#[no_mangle]
#[inline(always)]
fn verify_multisig_confirmation() -> i32 {
    unsafe {
        let tx_id = to_base58_string!(tx!(tx_id));
        let this = get_storage!(string::KEY_THIS);
        let multisig = get_storage!(string::KEY_MULTISIG);

        let status_key = join!(string::KEY_STATUS, SEP, this, SEP, tx_id);
        require!(
            contains_key!(base58!(multisig) => status_key)
                && get_storage!(boolean::base58!(multisig) => status_key),
            "verify_multisig_confirmation: revert"
        );
    }

    0
}

#[interface]
trait adapter {
    fn release_tokens(
        execution_chain_id: Integer,
        execution_asset: String,
        amount: Integer,
        recipient: String,
        gasless_reward: Integer,
        referrer: String,
        referrer_fee: Integer,
    );

    fn mint_tokens(
        execution_chain_id: Integer,
        execution_asset: String,
        amount: Integer,
        recipient: String,
        gasless_reward: Integer,
        referrer: String,
        referrer_fee: Integer,
    );
}

#[action]
fn _constructor(multisig: String, pauser: String) {
    require!(!contains_key!(KEY_INIT), "_constructor: already inited");
    require!(
        validate_contract(base58!(multisig)),
        "_constructor: inv multisig"
    );
    require!(
        validate_address(base58!(pauser)),
        "_constructor: inv pauser"
    );

    set_storage!(boolean::KEY_INIT => true);
    set_storage!(string::KEY_THIS => to_base58_string!(tx!(tx_id)));
    set_storage!(string::KEY_MULTISIG => multisig);
    set_storage!(boolean::KEY_PAUSED => false);
    set_storage!(string::KEY_PAUSER => pauser);
}

#[action]
fn set_adapter(execution_chain_id: Integer, adapter: String) {
    let exitcode = verify_multisig_confirmation();
    if exitcode != 0 {
        return exitcode;
    }

    require!(
        execution_chain_id >= 0,
        "set_adapter: inv execution_chain_id"
    );
    require!(
        validate_contract(base58!(adapter)),
        "set_adapter: inv adapter"
    );

    set_storage!(string::join!(string::KEY_ADAPTER, SEP, to_string_int!(execution_chain_id)) => adapter);
}

#[action]
fn release_tokens(
    execution_chain_id: Integer,
    execution_asset: String,
    amount: Integer,
    recipient: String,
    gasless_reward: Integer,
    referrer: String,
    referrer_fee: Integer,
) {
    let caller = to_base58_string!(caller!());
    require!(caller.len() > 0, "release_tokens: caller is not contract");

    let allowance_key = join!(string::KEY_ALLOWANCE, SEP, caller);
    require!(
        contains_key!(allowance_key) && get_storage!(boolean::allowance_key),
        "release_tokens: not allowed"
    );

    require!(!get_storage!(boolean::KEY_PAUSED), "release_tokens: paused");

    let adapter_key = join!(string::KEY_ADAPTER, SEP, to_string_int!(execution_chain_id));
    require!(contains_key!(adapter_key), "release_tokens: no adapter");

    let adapter = base58!(get_storage!(string::adapter_key));
    call_contract! {
        adapter(adapter)::release_tokens(execution_chain_id, execution_asset, amount, recipient, gasless_reward, referrer, referrer_fee)
    };
}

#[action]
fn mint_tokens(
    execution_chain_id: Integer,
    execution_asset: String,
    amount: Integer,
    recipient: String,
    gasless_reward: Integer,
    referrer: String,
    referrer_fee: Integer,
) {
    let caller = to_base58_string!(caller!());
    require!(caller.len() > 0, "mint_tokens: caller is not contract");

    let allowance_key = join!(string::KEY_ALLOWANCE, SEP, caller);
    require!(
        contains_key!(allowance_key) && get_storage!(boolean::allowance_key),
        "mint_tokens: not allowed"
    );

    require!(!get_storage!(boolean::KEY_PAUSED), "mint_tokens: paused");

    let adapter_key = join!(string::KEY_ADAPTER, SEP, to_string_int!(execution_chain_id));
    require!(contains_key!(adapter_key), "mint_tokens: no adapter");

    let adapter = base58!(get_storage!(string::adapter_key));
    call_contract! {
        adapter(adapter)::mint_tokens(execution_chain_id, execution_asset, amount, recipient, gasless_reward, referrer, referrer_fee)
    };
}

#[action]
fn allow(caller: String) {
    let exitcode = verify_multisig_confirmation();
    if exitcode != 0 {
        return exitcode;
    }

    require!(validate_contract(base58!(caller)), "allow: inv caller");

    set_storage!(boolean::join!(string::KEY_ALLOWANCE, SEP, caller) => true);
}

#[action]
fn disallow(caller: String) {
    let exitcode = verify_multisig_confirmation();
    if exitcode != 0 {
        return exitcode;
    }

    require!(validate_contract(base58!(caller)), "disallow: inv caller");

    set_storage!(boolean::join!(string::KEY_ALLOWANCE, SEP, caller) => false);
}

#[action]
fn pause() {
    let sender: String = to_base58_string!(tx!(sender));
    require!(to_base58_string!(caller!()).len() == 0);

    require!(
        equals!(string::sender, get_storage!(string::KEY_PAUSER)),
        "pause: not pauser"
    );
    require!(!get_storage!(boolean::KEY_PAUSED), "pause: paused");

    set_storage!(boolean::KEY_PAUSED => true);
}

#[action]
fn unpause() {
    let sender: String = to_base58_string!(tx!(sender));
    require!(to_base58_string!(caller!()).len() == 0);

    require!(
        equals!(string::sender, get_storage!(string::KEY_PAUSER)),
        "unpause: not pauser"
    );
    require!(get_storage!(boolean::KEY_PAUSED), "unpause: not paused");

    set_storage!(boolean::KEY_PAUSED => false);
}

#[action]
fn update_pauser(new_pauser: String) {
    let exitcode = verify_multisig_confirmation();
    if exitcode != 0 {
        return exitcode;
    }

    require!(
        validate_address(base58!(new_pauser)),
        "update_pauser: inv pauser"
    );

    set_storage!(string::KEY_PAUSER => new_pauser);
}

#[action]
fn update_multisig(new_multisig: String) {
    let exitcode = verify_multisig_confirmation();
    if exitcode != 0 {
        return exitcode;
    }

    require!(
        validate_contract(base58!(new_multisig)),
        "update_multisig: inv new_multisig"
    );

    set_storage!(string::KEY_MULTISIG => new_multisig);
}
