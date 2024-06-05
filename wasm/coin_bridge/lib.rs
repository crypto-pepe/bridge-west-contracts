#![no_std]
#![no_main]
// TODO: QA: TEST UNDERFLOW OVERFLOW AT ALL CONTRACTS
use we_cdk::*;

const SEP: String = "__";
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
) {
    require!(validate_contract(base58!(multisig)));
    require!(validate_contract(base58!(executor)));
    require!(validate_contract(base58!(adapter)));
    require!(validate_address(base58!(pauser)));
    require!(fee_recipient.len() > 0);

    set_storage!(string::KEY_THIS => to_base58_string!(tx!(tx_id)));
    set_storage!(string::KEY_MULTISIG => multisig);
    set_storage!(boolean::KEY_PAUSED => false);
    set_storage!(string::KEY_PAUSER => pauser);
    set_storage!(string::KEY_EXECUTOR => executor);
    set_storage!(string::KEY_ROOT_ADAPTER => adapter);
    set_storage!(integer::KEY_BALANCE => 0);
    set_storage!(integer::KEY_FEE => 0);
    set_storage!(string::KEY_FEE_RECIPIENT => fee_recipient);
}

#[action]
fn lock_tokens(
    execution_chain_id: Integer,
    recipient: String,
    referrer: String,
    gasless_reward: Integer,
) {
    require!(recipient.len() > 0);
    require!(gasless_reward >= 0);
    require!(!get_storage!(boolean::KEY_PAUSED));

    let chain_key = join!(string::KEY_CHAIN, SEP, to_string_int!(execution_chain_id));
    require!(contains_key!(chain_key) && get_storage!(boolean::chain_key));

    let payments_size = get_tx_payments!();
    require!(payments_size == 1);

    let (payment_asset, amount) = get_tx_payment!(0);
    require!(equals!(binary::payment_asset, SYSTEM_TOKEN));
    require!(amount > 0);

    let binding_key = join!(string::KEY_BINDING, SEP, to_string_int!(execution_chain_id));
    require!(contains_key!(binding_key));
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
            6 => enabled = parse_bool!(field),
            _ => {}
        }

        if index != -1 {
            binding_mut = drop!(binding_mut, index + SEP_SIZE);
        }

        binding_index += 1;
    }

    require!(execution_asset.len() > 0);
    require!(amount >= min_amount);
    require!(enabled);

    let percent = if amount > threshold_fee {
        after_percent_fee
    } else {
        before_percent_fee
    };

    let fee = min_fee + (amount * percent / PERCENT_FACTOR);
    require!(amount > fee);

    let mut referrer_fee_amount = 0;
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
        referrer_fee_amount = referrer_fee_percent * fee / PERCENT_FACTOR;
        if validate_address(base58!(referrer)) {
            transfer!(address => base58!(referrer), referrer_fee_amount);
        } else {
            transfer!(contract => base58!(referrer), referrer_fee_amount);
        }
    }

    let amount_to_send = amount - fee;
    require!(amount_to_send > gasless_reward);

    let amount_to_fee = fee - referrer_fee_amount;
    require!(amount_to_fee >= 0);

    let normalized_amount = normalize_decimals(amount_to_send, WEST_DECIMALS, DECIMALS);
    let normalized_gasless = normalize_decimals(gasless_reward, WEST_DECIMALS, DECIMALS);
    let normalized_referrer_fee = normalize_decimals(referrer_fee_amount, WEST_DECIMALS, DECIMALS);

    let root_adapter = base58!(get_storage!(string::KEY_ROOT_ADAPTER));
    call_contract! {
        root_adapter_contract(root_adapter)::mint_tokens(execution_chain_id, execution_asset, normalized_amount, recipient, normalized_gasless, referrer, normalized_referrer_fee)
    };

    set_storage!(integer::KEY_BALANCE => get_storage!(integer::KEY_BALANCE) + amount_to_send + referrer_fee_amount);
    set_storage!(integer::KEY_FEE => get_storage!(integer::KEY_FEE) + amount_to_fee);
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

    require!(!get_storage!(boolean::KEY_PAUSED));
    require!(caller.len() > 0);
    require!(equals!(string::caller, get_storage!(string::KEY_EXECUTOR)));
    require!(contains_key!(KEY_CALLER_CONTRACT));
    require!(caller_contract == get_storage!(string::KEY_CALLER_CONTRACT));
    require!(validate_address(base58!(recipient)) || validate_contract(base58!(recipient)));
    require!(amount > 0);
    require!(gasless_reward >= 0);

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
    require!(new_balance >= 0);

    set_storage!(integer::KEY_BALANCE => new_balance);
}

#[action]
fn transfer_fee(execution_chain_id: Integer) {
    require!(!get_storage!(boolean::KEY_PAUSED));

    let chain_key = join!(string::KEY_CHAIN, SEP, to_string_int!(execution_chain_id));
    require!(contains_key!(chain_key) && get_storage!(boolean::chain_key));

    let binding_key = join!(string::KEY_BINDING, SEP, to_string_int!(execution_chain_id));
    require!(contains_key!(binding_key));
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
            6 => enabled = parse_bool!(field),
            _ => {}
        }

        if index != -1 {
            binding_mut = drop!(binding_mut, index + SEP_SIZE);
        }

        binding_index += 1;
    }

    let fee_amount = get_storage!(integer::KEY_FEE);

    require!(execution_asset.len() > 0);
    require!(fee_amount >= min_amount);
    require!(enabled);

    let normalized_fee_amount = normalize_decimals(fee_amount, WEST_DECIMALS, DECIMALS);

    let root_adapter = base58!(get_storage!(string::KEY_ROOT_ADAPTER));
    call_contract! {
        root_adapter_contract(root_adapter)::mint_tokens(execution_chain_id, execution_asset, normalized_fee_amount, get_storage!(string::KEY_FEE_RECIPIENT), 0, "", 0)
    };

    set_storage!(integer::KEY_BALANCE => get_storage!(integer::KEY_BALANCE) + fee_amount);
    set_storage!(integer::KEY_FEE => 0);
}

#[action]
fn update_caller_contract(caller_contract: String) {
    let tx_id = to_base58_string!(tx!(tx_id));
    let this = get_storage!(string::KEY_THIS);
    let multisig = get_storage!(string::KEY_MULTISIG);

    let status_key = join!(string::KEY_STATUS, SEP, this, SEP, tx_id);
    require!(
        contains_key!(base58!(multisig) => status_key)
            && get_storage!(boolean::base58!(multisig) => status_key)
    );

    require!(caller_contract.len() > 0);

    set_storage!(string::KEY_CALLER_CONTRACT => caller_contract);
}

#[action]
fn update_execution_chain(execution_chain_id: Integer, enabled: Boolean) {
    let tx_id = to_base58_string!(tx!(tx_id));
    let this = get_storage!(string::KEY_THIS);
    let multisig = get_storage!(string::KEY_MULTISIG);

    let status_key = join!(string::KEY_STATUS, SEP, this, SEP, tx_id);
    require!(
        contains_key!(base58!(multisig) => status_key)
            && get_storage!(boolean::base58!(multisig) => status_key)
    );

    require!(execution_chain_id >= 0);

    set_storage!(boolean::join!(
        string::KEY_CHAIN,
        SEP,
        to_string_int!(execution_chain_id)
    ) => enabled);
}

#[action]
fn update_fee_recipient(fee_recipient: String) {
    let tx_id = to_base58_string!(tx!(tx_id));
    let this = get_storage!(string::KEY_THIS);
    let multisig = get_storage!(string::KEY_MULTISIG);

    let status_key = join!(string::KEY_STATUS, SEP, this, SEP, tx_id);
    require!(
        contains_key!(base58!(multisig) => status_key)
            && get_storage!(boolean::base58!(multisig) => status_key)
    );

    require!(fee_recipient.len() > 0);

    set_storage!(string::KEY_FEE_RECIPIENT => fee_recipient);
}

#[action]
fn update_referrer(execution_chain_id: Integer, referrer: String, fee: Integer) {
    let tx_id = to_base58_string!(tx!(tx_id));
    let this = get_storage!(string::KEY_THIS);
    let multisig = get_storage!(string::KEY_MULTISIG);

    let status_key = join!(string::KEY_STATUS, SEP, this, SEP, tx_id);
    require!(
        contains_key!(base58!(multisig) => status_key)
            && get_storage!(boolean::base58!(multisig) => status_key)
    );

    let chain_key = join!(string::KEY_CHAIN, SEP, to_string_int!(execution_chain_id));
    require!(contains_key!(chain_key) && get_storage!(boolean::chain_key));

    require!(referrer.len() > 0);
    require!(fee >= 0 && fee <= MAX_REFERRER_FEE);

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
    let tx_id = to_base58_string!(tx!(tx_id));
    let this = get_storage!(string::KEY_THIS);
    let multisig = get_storage!(string::KEY_MULTISIG);

    let status_key = join!(string::KEY_STATUS, SEP, this, SEP, tx_id);
    require!(
        contains_key!(base58!(multisig) => status_key)
            && get_storage!(boolean::base58!(multisig) => status_key)
    );

    require!(execution_chain_id >= 0);
    require!(execution_asset.len() > 0);
    require!(min_amount >= 0);
    require!(min_fee >= 0);
    require!(threshold_fee >= 0);
    require!(before_percent_fee >= 0);
    require!(after_percent_fee >= 0);

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
        to_string_bool!(enabled)
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

    require!(equals!(string::sender, get_storage!(string::KEY_PAUSER)));
    require!(!get_storage!(boolean::KEY_PAUSED));

    set_storage!(boolean::KEY_PAUSED => true);
}

#[action]
fn unpause() {
    let sender: String = to_base58_string!(tx!(sender));

    require!(equals!(string::sender, get_storage!(string::KEY_PAUSER)));
    require!(get_storage!(boolean::KEY_PAUSED));

    set_storage!(boolean::KEY_PAUSED => false);
}

#[action]
fn update_pauser(new_pauser: String) {
    let tx_id = to_base58_string!(tx!(tx_id));
    let this = get_storage!(string::KEY_THIS);
    let multisig = get_storage!(string::KEY_MULTISIG);

    let status_key = join!(string::KEY_STATUS, SEP, this, SEP, tx_id);
    require!(
        contains_key!(base58!(multisig) => status_key)
            && get_storage!(boolean::base58!(multisig) => status_key)
    );

    require!(validate_address(base58!(new_pauser)));

    set_storage!(string::KEY_PAUSER => new_pauser);
}

#[action]
fn update_multisig(new_multisig: String) {
    let tx_id = to_base58_string!(tx!(tx_id));
    let this = get_storage!(string::KEY_THIS);
    let multisig = get_storage!(string::KEY_MULTISIG);

    let status_key = join!(string::KEY_STATUS, SEP, this, SEP, tx_id);
    require!(
        contains_key!(base58!(multisig) => status_key)
            && get_storage!(boolean::base58!(multisig) => status_key)
    );

    require!(validate_contract(base58!(new_multisig)));

    set_storage!(string::KEY_MULTISIG => new_multisig);
}
