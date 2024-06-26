#![no_std]
#![no_main]

use we_cdk::*;

const SEP: String = "__";
const FUNC_SEP: String = "####";
const KEY_INIT: String = "INIT";
const KEY_THIS: String = "THIS";
const KEY_MULTISIG: String = "MULTISIG";
const KEY_STATUS: String = "STATUS";
const KEY_PAUSED: String = "PAUSED";
const KEY_PAUSER: String = "PAUSER";
const KEY_ROOT_ADAPTER: String = "ROOT_ADAPTER";
const KEY_PROTOCOL_CALLER: String = "PROTOCOL_CALLER";
const KEY_COIN_BRIDGE_CONTRACT: String = "COIN_BRIDGE_CONTRACT";
const KEY_TOKEN_BRIDGE_CONTRACT: String = "TOKEN_BRIDGE_CONTRACT";
const KEY_WRAPPED_TOKEN_BRIDGE_CONTRACT: String = "WRAPPED_TOKEN_BRIDGE_CONTRACT";

const FUNC_MINT_TOKENS: String = "mintTokens";
const FUNC_RELEASE_TOKENS: String = "releaseTokens";

const WAVES: String = "WAVES";

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
trait caller {
    fn call(
        execution_chain_id: Integer,
        execution_contract: String,
        function_name: String,
        function_args: String,
    );
}

#[action]
fn _constructor(multisig: String, protocol_caller: String, root_adapter: String, pauser: String) {
    require!(!contains_key!(KEY_INIT), "_constructor: already inited");
    require!(
        validate_contract(base58!(multisig)),
        "_constructor: inv multisig"
    );
    require!(
        validate_contract(base58!(protocol_caller)),
        "_constructor: inv protocol_caller"
    );
    require!(
        validate_contract(base58!(root_adapter)),
        "_constructor: inv root_adapter"
    );
    require!(
        validate_address(base58!(pauser)),
        "_constructor: inv pauser"
    );

    set_storage!(boolean::KEY_INIT => true);
    set_storage!(string::KEY_THIS => to_base58_string!(tx!(tx_id)));
    set_storage!(string::KEY_MULTISIG => multisig);
    set_storage!(string::KEY_PROTOCOL_CALLER => protocol_caller);
    set_storage!(string::KEY_ROOT_ADAPTER => root_adapter);
    set_storage!(boolean::KEY_PAUSED => false);
    set_storage!(string::KEY_PAUSER => pauser);
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

    require!(!get_storage!(boolean::KEY_PAUSED), "release_tokens: paused");
    require!(
        equals!(string::caller, get_storage!(string::KEY_ROOT_ADAPTER)),
        "release_tokens: inv caller"
    );

    if execution_asset == WAVES {
        require!(
            contains_key!(KEY_COIN_BRIDGE_CONTRACT),
            "release_tokens: coin bridge contract not set"
        );
        let execution_contract = get_storage!(string::KEY_COIN_BRIDGE_CONTRACT);
        let args = join!(
            string::to_string_int!(amount),
            FUNC_SEP,
            recipient,
            FUNC_SEP,
            to_string_int!(gasless_reward),
            FUNC_SEP,
            referrer,
            FUNC_SEP,
            to_string_int!(referrer_fee)
        );

        call_contract! {
            caller(base58!(get_storage!(string::KEY_PROTOCOL_CALLER)))::call(execution_chain_id, execution_contract, FUNC_RELEASE_TOKENS, args.as_ref())
        };
    } else {
        require!(
            contains_key!(KEY_TOKEN_BRIDGE_CONTRACT),
            "release_tokens: token bridge contract not set"
        );
        let execution_contract = get_storage!(string::KEY_TOKEN_BRIDGE_CONTRACT);
        let args = join!(
            string::execution_asset,
            FUNC_SEP,
            to_string_int!(amount),
            FUNC_SEP,
            recipient,
            FUNC_SEP,
            to_string_int!(gasless_reward),
            FUNC_SEP,
            referrer,
            FUNC_SEP,
            to_string_int!(referrer_fee)
        );

        call_contract! {
            caller(base58!(get_storage!(string::KEY_PROTOCOL_CALLER)))::call(execution_chain_id, execution_contract, FUNC_RELEASE_TOKENS, args.as_ref())
        };
    }
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

    require!(!get_storage!(boolean::KEY_PAUSED), "mint_tokens: paused");
    require!(
        equals!(string::caller, get_storage!(string::KEY_ROOT_ADAPTER)),
        "mint_tokens: inv caller"
    );

    require!(
        contains_key!(KEY_WRAPPED_TOKEN_BRIDGE_CONTRACT),
        "mint_tokens: wrapped token bridge contract not set"
    );
    let execution_contract = get_storage!(string::KEY_WRAPPED_TOKEN_BRIDGE_CONTRACT);
    let args = join!(
        string::execution_asset,
        FUNC_SEP,
        to_string_int!(amount),
        FUNC_SEP,
        recipient,
        FUNC_SEP,
        to_string_int!(gasless_reward),
        FUNC_SEP,
        referrer,
        FUNC_SEP,
        to_string_int!(referrer_fee)
    );

    call_contract! {
        caller(base58!(get_storage!(string::KEY_PROTOCOL_CALLER)))::call(execution_chain_id, execution_contract, FUNC_MINT_TOKENS, args)
    };
}

#[action]
fn set_coin_bridge_contract(contract: String) {
    let exitcode = verify_multisig_confirmation();
    if exitcode != 0 {
        return exitcode;
    }

    require!(contract.len() > 0, "set_coin_bridge_contract: inv contract");

    set_storage!(string::KEY_COIN_BRIDGE_CONTRACT => contract);
}

#[action]
fn set_token_bridge_contract(contract: String) {
    let exitcode = verify_multisig_confirmation();
    if exitcode != 0 {
        return exitcode;
    }

    require!(
        contract.len() > 0,
        "set_token_bridge_contract: inv contract"
    );

    set_storage!(string::KEY_TOKEN_BRIDGE_CONTRACT => contract);
}

#[action]
fn set_wrapped_token_bridge_contract(contract: String) {
    let exitcode = verify_multisig_confirmation();
    if exitcode != 0 {
        return exitcode;
    }

    require!(
        contract.len() > 0,
        "set_wrapped_token_bridge_contract: inv contract"
    );

    set_storage!(string::KEY_WRAPPED_TOKEN_BRIDGE_CONTRACT => contract);
}

#[action]
fn pause() {
    let sender: String = to_base58_string!(tx!(sender));

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
