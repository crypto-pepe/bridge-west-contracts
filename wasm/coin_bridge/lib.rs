#![no_std]
#![no_main]
// TODO: QA: TEST UNDERFLOW OVERFLOW AT ALL CONTRACTS
use we_cdk::*;

const SEP: String = "__";
const KEY_INIT: String = "INIT";
const KEY_THIS: String = "THIS";
const KEY_MULTISIG: String = "MULTISIG";
const KEY_STATUS: String = "STATUS";
const KEY_PAUSED: String = "PAUSED";
const KEY_PAUSER: String = "PAUSER";
const KEY_EXECUTOR: String = "EXECUTOR";
const KEY_ROOT_ADAPTER: String = "ROOT_ADAPTER";
const KEY_CALLER_CONTRACT: String = "CALLER_CONTRACT";
const KEY_CHAIN: String = "CHAIN";
const KEY_BINDING: String = "BINDING";
const KEY_FEE: String = "FEE";
const KEY_BALANCE: String = "BALANCE";
const KEY_FEE_RECIPIENT: String = "FEE_RECIPIENT";
const KEY_FEE_CHAIN: String = "FEE_CHAIN";
const KEY_REFERRER_FEE_PERCENT: String = "REFERRER_FEE_PERCENT";

const SEP_SIZE: Integer = 2;
const BINDING_FIELD_SIZE: Integer = 7;
const PERCENT_FACTOR: Integer = 1000000;
const WEST_DECIMALS: Integer = 8;
const DECIMALS: Integer = 6;
const MAX_REFERRER_FEE: Integer = 200000; // Referrer fee is up to 20%

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

fn pow_10(pow: i64) -> i64 {
    match pow {
        0 => 1,
        2 => 100,
        3 => 1000,
        4 => 10000,
        5 => 100000,
        6 => 1000000,
        7 => 10000000,
        8 => 100000000,
        9 => 1000000000,
        10 => 10000000000,
        _ => 0,
    }
}

fn normalize_decimals(
    amount: Integer,
    source_decimals: Integer,
    target_decimals: Integer,
) -> Integer {
    if source_decimals >= target_decimals {
        let delimiter: i64 = pow_10(source_decimals - target_decimals);
        amount / delimiter
    } else {
        let multiplier: i64 = pow_10(target_decimals - source_decimals);
        amount * multiplier
    }
}

#[interface]
trait root_adapter_contract {
    fn mint_tokens(
        execution_chain_id: Integer,
        execution_asset: String,
        amount: Integer,
        recipient: String,
        gasless_amount: Integer,
        referrer: String,
        referrer_fee: Integer,
    );
}

#[action]
fn _constructor(
    multisig: String,
    executor: String,
    adapter: String,
    pauser: String,
    fee_recipient: String,
    fee_chain_id: Integer,
    caller_contract: String,
) {
    require!(!contains_key!(KEY_INIT), "_constructor: already inited");
    require!(
        validate_contract(base58!(multisig)),
        "_constructor: inv multisig"
    );
    require!(
        validate_contract(base58!(executor)),
        "_constructor: inv executor"
    );
    require!(
        validate_contract(base58!(adapter)),
        "_constructor: inv adapter"
    );
    require!(
        validate_address(base58!(pauser)),
        "_constructor: inv pauser"
    );
    require!(fee_recipient.len() > 0, "_constructor: inv fee_recipient");
    require!(fee_chain_id > 0, "_constructor: inv fee_chain_id");
    require!(
        caller_contract.len() > 0,
        "_constructor: inv caller contract"
    );

    set_storage!(boolean::KEY_INIT => true);
    set_storage!(string::KEY_THIS => to_base58_string!(tx!(tx_id)));
    set_storage!(string::KEY_MULTISIG => multisig);
    set_storage!(boolean::KEY_PAUSED => false);
    set_storage!(string::KEY_PAUSER => pauser);
    set_storage!(string::KEY_EXECUTOR => executor);
    set_storage!(string::KEY_ROOT_ADAPTER => adapter);
    set_storage!(integer::KEY_BALANCE => 0);
    set_storage!(integer::KEY_FEE => 0);
    set_storage!(string::KEY_FEE_RECIPIENT => fee_recipient);
    set_storage!(integer::KEY_FEE_CHAIN => fee_chain_id);
    set_storage!(string::KEY_CALLER_CONTRACT => caller_contract);
}

#[action]
fn lock_tokens(
    execution_chain_id: Integer,
    recipient: String,
    referrer: String,
    gasless_reward: Integer,
) {
    require!(recipient.len() > 0, "lock_tokens: inv recipient");
    require!(gasless_reward >= 0, "lock_tokens: inv gasless_reward");
    require!(!get_storage!(boolean::KEY_PAUSED), "lock_tokens: paused");

    let chain_key = join!(string::KEY_CHAIN, SEP, to_string_int!(execution_chain_id));
    require!(
        contains_key!(chain_key) && get_storage!(boolean::chain_key),
        "lock_tokens: chain disabled"
    );

    let payments_size = get_tx_payments!();
    require!(payments_size == 1, "lock_tokens: no payments");

    let (payment_asset, amount) = get_tx_payment!(0);
    require!(
        equals!(binary::payment_asset, SYSTEM_TOKEN),
        "lock_tokens: payment is not WEST"
    );
    require!(amount > 0, "lock_tokens: inv amount");

    let binding_key = join!(string::KEY_BINDING, SEP, to_string_int!(execution_chain_id));
    require!(contains_key!(binding_key), "lock_tokens: no binding");
    let binding_raw = get_storage!(string::binding_key);

    let mut binding_mut = binding_raw.as_bytes();
    let mut binding_index = 0;
    let mut execution_asset = "";
    let mut min_amount = 0;
    let mut min_fee = 0;
    let mut threshold_fee = 0;
    let mut before_percent_fee = 0;
    let mut after_percent_fee = 0;
    let mut enabled = false;

    while binding_index < BINDING_FIELD_SIZE {
        let index = index_of!(binding_mut, SEP);

        let field = if index == -1 {
            binding_mut
        } else {
            take!(binding_mut, index)
        };

        match binding_index {
            0 => execution_asset = core::str::from_utf8_unchecked(field),
            1 => min_amount = parse_int!(field),
            2 => min_fee = parse_int!(field),
            3 => threshold_fee = parse_int!(field),
            4 => before_percent_fee = parse_int!(field),
            5 => after_percent_fee = parse_int!(field),
            6 => {
                enabled = match parse_int!(field) {
                    0 => false,
                    _ => true,
                }
            }
            _ => {}
        }

        if index != -1 {
            binding_mut = drop!(binding_mut, index + SEP_SIZE);
        }

        binding_index += 1;
    }

    require!(execution_asset.len() > 0, "lock_tokens: inv binding");
    require!(amount >= min_amount, "lock_tokens: less than min");
    require!(enabled, "lock_tokens: binding disabled");

    let percent = if amount > threshold_fee {
        after_percent_fee
    } else {
        before_percent_fee
    };

    let fee = min_fee + (amount * percent / PERCENT_FACTOR);
    require!(amount > fee, "lock_tokens: fee less than amount");

    let mut referrer_fee = 0;
    if referrer.len() > 0 {
        let referrer_fee_percent_key = join!(
            string::KEY_REFERRER_FEE_PERCENT,
            SEP,
            to_string_int!(execution_chain_id),
            SEP,
            referrer
        );
        let referrer_fee_percent = if contains_key!(referrer_fee_percent_key) {
            get_storage!(integer::referrer_fee_percent_key)
        } else {
            0
        };
        referrer_fee = referrer_fee_percent * fee / PERCENT_FACTOR;
    }

    let amount_to_send = amount - fee;
    require!(
        amount_to_send > gasless_reward,
        "lock_tokens: amount less than gasless_reward"
    );

    let normalized_amount = normalize_decimals(amount_to_send, WEST_DECIMALS, DECIMALS);
    let normalized_gasless = normalize_decimals(gasless_reward, WEST_DECIMALS, DECIMALS);
    let normalized_referrer_fee = normalize_decimals(referrer_fee, WEST_DECIMALS, DECIMALS);

    let root_adapter = base58!(get_storage!(string::KEY_ROOT_ADAPTER));
    call_contract! {
        root_adapter_contract(root_adapter)::mint_tokens(execution_chain_id, execution_asset, normalized_amount, recipient, normalized_gasless, referrer, normalized_referrer_fee)
    };

    set_storage!(integer::KEY_BALANCE => get_storage!(integer::KEY_BALANCE) + amount_to_send + referrer_fee);
    set_storage!(integer::KEY_FEE => get_storage!(integer::KEY_FEE) + fee - referrer_fee);
}

#[action]
fn release_tokens(
    caller_contract: String,
    recipient: String,
    amount: String,
    gasless_reward: String,
) {
    let amount: Integer = parse_int!(amount);
    let gasless_reward: Integer = parse_int!(gasless_reward);
    let sender: String = to_base58_string!(tx!(sender));
    let caller: String = to_base58_string!(caller!());

    require!(!get_storage!(boolean::KEY_PAUSED), "release_tokens: paused");
    require!(caller.len() > 0, "release_tokens: caller is not contract");
    require!(
        equals!(string::caller, get_storage!(string::KEY_EXECUTOR)),
        "release_tokens: only executor"
    );
    require!(
        contains_key!(KEY_CALLER_CONTRACT),
        "release_tokens: no caller contract key"
    );
    require!(
        caller_contract == get_storage!(string::KEY_CALLER_CONTRACT),
        "release_tokens: inv caller contract"
    );
    require!(
        validate_address(base58!(recipient)) || validate_contract(base58!(recipient)),
        "release_tokens: inv recipient"
    );
    require!(amount > 0, "release_tokens: inv amount");
    require!(gasless_reward >= 0, "release_tokens: inv gasless_reward");

    let normalized_amount = normalize_decimals(amount, DECIMALS, WEST_DECIMALS);
    let normalized_gasless = normalize_decimals(gasless_reward, DECIMALS, WEST_DECIMALS);

    if normalized_gasless > 0 && !equals!(string::recipient, sender) {
        if validate_address(base58!(recipient)) {
            transfer!(address => base58!(recipient), normalized_amount - normalized_gasless);
        } else {
            transfer!(contract => base58!(recipient), normalized_amount - normalized_gasless);
        }
        transfer!(address => base58!(sender), normalized_gasless);
    } else {
        if validate_address(base58!(recipient)) {
            transfer!(address => base58!(recipient), normalized_amount);
        } else {
            transfer!(contract => base58!(recipient), normalized_amount);
        }
    }

    let new_balance = get_storage!(integer::KEY_BALANCE) - normalized_amount;
    require!(new_balance >= 0, "release_tokens: new_balance < 0");

    set_storage!(integer::KEY_BALANCE => new_balance);
}

#[action]
fn transfer_fee() {
    require!(!get_storage!(boolean::KEY_PAUSED), "transfer_fee: paused");

    let fee_chain_id = get_storage!(integer::KEY_FEE_CHAIN);
    let chain_key = join!(string::KEY_CHAIN, SEP, to_string_int!(fee_chain_id));
    require!(
        contains_key!(chain_key) && get_storage!(boolean::chain_key),
        "transfer_fee: disabled chain"
    );

    let binding_key = join!(string::KEY_BINDING, SEP, to_string_int!(fee_chain_id));
    require!(contains_key!(binding_key), "transfer_fee: no binding");
    let binding_raw = get_storage!(string::binding_key);

    let mut binding_mut = binding_raw.as_bytes();
    let mut binding_index = 0;
    let mut execution_asset = "";
    let mut min_amount = 0;
    let mut _min_fee = 0;
    let mut _threshold_fee = 0;
    let mut _before_percent_fee = 0;
    let mut _after_percent_fee = 0;
    let mut enabled = false;

    while binding_index < BINDING_FIELD_SIZE {
        let index = index_of!(binding_mut, SEP);

        let field = if index == -1 {
            binding_mut
        } else {
            take!(binding_mut, index)
        };

        match binding_index {
            0 => execution_asset = core::str::from_utf8_unchecked(field),
            1 => min_amount = parse_int!(field),
            2 => _min_fee = parse_int!(field),
            3 => _threshold_fee = parse_int!(field),
            4 => _before_percent_fee = parse_int!(field),
            5 => _after_percent_fee = parse_int!(field),
            6 => {
                enabled = {
                    match parse_int!(field) {
                        0 => false,
                        _ => true,
                    }
                }
            }
            _ => {}
        }

        if index != -1 {
            binding_mut = drop!(binding_mut, index + SEP_SIZE);
        }

        binding_index += 1;
    }

    let fee_amount = get_storage!(integer::KEY_FEE);

    require!(execution_asset.len() > 0, "transfer_fee: inv binding");
    require!(fee_amount >= min_amount, "transfer_fee: less than min");
    require!(enabled, "transfer_fee: binding disabled");

    let normalized_fee_amount = normalize_decimals(fee_amount, WEST_DECIMALS, DECIMALS);

    let root_adapter = base58!(get_storage!(string::KEY_ROOT_ADAPTER));
    call_contract! {
        root_adapter_contract(root_adapter)::mint_tokens(fee_chain_id, execution_asset, normalized_fee_amount, get_storage!(string::KEY_FEE_RECIPIENT), 0, "", 0)
    };

    set_storage!(integer::KEY_BALANCE => get_storage!(integer::KEY_BALANCE) + fee_amount);
    set_storage!(integer::KEY_FEE => 0);
}

#[action]
fn update_caller_contract(caller_contract: String) {
    let exitcode = verify_multisig_confirmation();
    if exitcode != 0 {
        return exitcode;
    }

    require!(
        caller_contract.len() > 0,
        "update_caller_contract: inv caller contract"
    );

    set_storage!(string::KEY_CALLER_CONTRACT => caller_contract);
}

#[action]
fn update_execution_chain(execution_chain_id: Integer, enabled: Boolean) {
    let exitcode = verify_multisig_confirmation();
    if exitcode != 0 {
        return exitcode;
    }

    require!(
        execution_chain_id >= 0,
        "update_execution_chain: inv execution_chain_id"
    );

    set_storage!(boolean::join!(
        string::KEY_CHAIN,
        SEP,
        to_string_int!(execution_chain_id)
    ) => enabled);
}

#[action]
fn update_fee_recipient(fee_recipient: String) {
    let exitcode = verify_multisig_confirmation();
    if exitcode != 0 {
        return exitcode;
    }

    require!(
        fee_recipient.len() > 0,
        "update_fee_recipient: inv fee_recipient"
    );

    set_storage!(string::KEY_FEE_RECIPIENT => fee_recipient);
}

#[action]
fn update_fee_chain(fee_chain: Integer) {
    let exitcode = verify_multisig_confirmation();
    if exitcode != 0 {
        return exitcode;
    }

    require!(fee_chain > 0, "update_fee_chain: inv fee_chain");

    set_storage!(integer::KEY_FEE_CHAIN => fee_chain);
}

#[action]
fn update_referrer(execution_chain_id: Integer, referrer: String, fee: Integer) {
    let exitcode = verify_multisig_confirmation();
    if exitcode != 0 {
        return exitcode;
    }

    let chain_key = join!(string::KEY_CHAIN, SEP, to_string_int!(execution_chain_id));
    require!(
        contains_key!(chain_key) && get_storage!(boolean::chain_key),
        "update_referrer: disabled chain"
    );

    require!(referrer.len() > 0, "update_referrer: inv referrer");
    require!(
        fee >= 0 && fee <= MAX_REFERRER_FEE,
        "update_referrer: inv fee"
    );

    set_storage!(integer::join!(
        string::KEY_REFERRER_FEE_PERCENT,
        SEP,
        to_string_int!(execution_chain_id),
        SEP,
        referrer
    ) => fee);
}

#[action]
fn update_binding_info(
    execution_chain_id: Integer,
    execution_asset: String,
    min_amount: Integer,
    min_fee: Integer,
    threshold_fee: Integer,
    before_percent_fee: Integer,
    after_percent_fee: Integer,
    enabled: Boolean,
) {
    let exitcode = verify_multisig_confirmation();
    if exitcode != 0 {
        return exitcode;
    }

    require!(
        execution_chain_id >= 0,
        "update_binding_info: inv execution_chain_id"
    );
    require!(
        execution_asset.len() > 0,
        "update_binding_info: inv execution_asset"
    );
    require!(min_amount >= 0, "update_binding_info: inv min_amount");
    require!(min_fee >= 0, "update_binding_info: inv min_fee");
    require!(threshold_fee >= 0, "update_binding_info: inv threshold_fee");
    require!(
        before_percent_fee >= 0,
        "update_binding_info: inv before_percent_fee"
    );
    require!(
        after_percent_fee >= 0,
        "update_binding_info: inv after_percent_fee"
    );

    let binding = join!(
        string::execution_asset,
        SEP,
        to_string_int!(min_amount),
        SEP,
        to_string_int!(min_fee),
        SEP,
        to_string_int!(threshold_fee),
        SEP,
        to_string_int!(before_percent_fee),
        SEP,
        to_string_int!(after_percent_fee),
        SEP,
        to_string_int!(if enabled { 1 } else { 0 })
    );

    set_storage!(string::join!(
        string::KEY_BINDING,
        SEP,
        to_string_int!(execution_chain_id)
    ) => binding);
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
