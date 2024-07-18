#![no_std]
#![no_main]

use we_cdk::*;

const SEP: String = "__";
const KEY_INIT: String = "INIT";
const KEY_THIS: String = "THIS";
const KEY_MULTISIG: String = "MULTISIG";
const KEY_STATUS: String = "STATUS";
const KEY_PUBLIC_KEYS: String = "PUBLIC_KEYS";
const KEY_QUORUM: String = "QUORUM";
const KEY_CONFIRM: String = "CONFIRM";

const SEP_SIZE: i64 = 2;

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

#[action]
fn _constructor(owners: String, quorum: Integer) {
    require!(!contains_key!(KEY_INIT), "_constructor: already inited");

    let mut owners_mut = owners.as_bytes();
    let mut owners_size = 0;

    loop {
        let index = index_of!(owners_mut, SEP);
        if index == -1 {
            let owner = owners_mut;
            require!(validate_contract(base58!(owner)), "_constructor: inv owner");
            owners_size += 1;
            break;
        }

        let owner = take!(owners_mut, index);
        owners_mut = drop!(owners_mut, index + SEP_SIZE);

        require!(
            !contains!(owners_mut, owner),
            "_constructor: duplicate owner"
        );
        require!(validate_contract(base58!(owner)), "_constructor: inv owner");
        owners_size += 1;
    }

    require!(quorum > 0, "_constructor: inv quorum > 0");
    require!(
        quorum <= owners_size,
        "_constructor: inv quorum <= owners_size"
    );

    set_storage!(boolean::KEY_INIT => true);
    set_storage!(string::KEY_THIS => to_base58_string!(tx!(tx_id)));
    set_storage!(string::KEY_MULTISIG => to_base58_string!(tx!(tx_id)));
    set_storage!(string::KEY_PUBLIC_KEYS => owners);
    set_storage!(integer::KEY_QUORUM => quorum);
}

#[action]
fn add_owner(new_owner: String) {
    let owners = get_storage!(string::KEY_PUBLIC_KEYS);

    let exitcode = verify_multisig_confirmation();
    if exitcode != 0 {
        return exitcode;
    }

    require!(
        validate_contract(base58!(new_owner)),
        "add_owner: inv new_owner"
    );
    require!(!contains!(owners, new_owner), "add_owner: already exists");

    let owners_updated = join!(string::owners, SEP, new_owner);
    set_storage!(string::KEY_PUBLIC_KEYS => owners_updated);
}

#[action]
fn remove_owner(old_owner: String) {
    let owners = get_storage!(string::KEY_PUBLIC_KEYS);
    let quorum: Integer = get_storage!(integer::KEY_QUORUM);

    let exitcode = verify_multisig_confirmation();
    if exitcode != 0 {
        return exitcode;
    }

    require!(
        validate_contract(base58!(old_owner)),
        "remove_owner: inv old_owner"
    );
    require!(
        contains!(owners, old_owner),
        "remove_owner: old_owner not exist"
    );

    let mut owners_mut = owners.as_bytes();
    let mut owners_size = 0;
    loop {
        let index = index_of!(owners_mut, SEP);
        if index == -1 {
            owners_size += 1;
            break;
        }

        owners_mut = drop!(owners_mut, index + SEP_SIZE);
        owners_size += 1;
    }
    require!(owners_size > 1, "remove_owner: no owners left");

    owners_mut = owners.as_bytes();
    let index = index_of!(owners_mut, old_owner);
    if index == 0 {
        owners_mut = drop!(owners_mut, index + SEP_SIZE);
    } else {
        let take_from_right = owners_mut.len() as i64 - index - old_owner.len() as i64;
        owners_mut = join!(
            binary::take!(owners_mut, index - SEP_SIZE),
            take_right!(owners_mut, take_from_right)
        );
    }

    if owners_size - 1 < quorum {
        set_storage!(integer::KEY_QUORUM => owners_size - 1);
    }

    set_storage!(string::KEY_PUBLIC_KEYS => owners_mut);
}

#[action]
fn set_quorum(quorum: Integer) {
    let owners = get_storage!(string::KEY_PUBLIC_KEYS);

    let exitcode = verify_multisig_confirmation();
    if exitcode != 0 {
        return exitcode;
    }

    let mut owners_mut = owners.as_bytes();
    let mut owners_size = 0;
    loop {
        let index = index_of!(owners_mut, SEP);
        if index == -1 {
            owners_size += 1;
            break;
        }

        owners_mut = drop!(owners_mut, index + SEP_SIZE);
        owners_size += 1;
    }

    require!(quorum > 0, "set_quorum: inv quorum > 0");
    require!(
        quorum <= owners_size,
        "set_quorum: inv quorum <= owners_size"
    );

    set_storage!(integer::KEY_QUORUM => quorum);
}

#[action]
fn confirm_transaction(dapp: String, tx_id: String) {
    let sender_public_key = to_base58_string!(tx!(sender_public_key));
    require!(to_base58_string!(caller!()).len() == 0);

    let owners = get_storage!(string::KEY_PUBLIC_KEYS);
    let quorum: Integer = get_storage!(integer::KEY_QUORUM);

    let confirmations_key = join!(string::KEY_CONFIRM, SEP, dapp, SEP, tx_id);
    let confirmations = if contains_key!(confirmations_key) {
        get_storage!(string::confirmations_key)
    } else {
        ""
    };

    let status_key = join!(string::KEY_STATUS, SEP, dapp, SEP, tx_id);
    let status = if contains_key!(status_key) {
        get_storage!(boolean::status_key)
    } else {
        false
    };

    require!(
        validate_contract(base58!(dapp)),
        "confirm_transaction: inv dapp"
    );
    require!(
        validate_contract(base58!(tx_id)),
        "confirm_transaction: inv tx_id"
    );
    require!(
        contains!(owners, sender_public_key),
        "confirm_transaction: inv caller"
    );
    require!(
        !contains!(confirmations, sender_public_key),
        "confirm_transaction: already confirmed by caller"
    );
    require!(!status, "confirm_transaction: already confirmed");

    let confirmations_updated;
    if confirmations.len() == 0 {
        confirmations_updated = join!(string::sender_public_key);
    } else {
        confirmations_updated = join!(string::confirmations, SEP, sender_public_key);
    }

    let mut confirmations_mut = confirmations_updated.as_bytes();
    let mut confirmations_size = 0;
    loop {
        let index = index_of!(confirmations_mut, SEP);
        if index == -1 {
            confirmations_size += 1;
            break;
        }

        confirmations_mut = drop!(confirmations_mut, index + SEP_SIZE);
        confirmations_size += 1;
    }

    set_storage!(string::confirmations_key => confirmations_updated);
    set_storage!(boolean::status_key => confirmations_size >= quorum);
}

#[action]
fn revoke_confirmation(dapp: String, tx_id: String) {
    let sender_public_key = to_base58_string!(tx!(sender_public_key));
    require!(to_base58_string!(caller!()).len() == 0);

    let owners = get_storage!(string::KEY_PUBLIC_KEYS);
    let quorum: Integer = get_storage!(integer::KEY_QUORUM);

    let confirmations_key = join!(string::KEY_CONFIRM, SEP, dapp, SEP, tx_id);
    let confirmations = if contains_key!(confirmations_key) {
        get_storage!(string::confirmations_key)
    } else {
        ""
    };

    let status_key = join!(string::KEY_STATUS, SEP, dapp, SEP, tx_id);
    let status = if contains_key!(status_key) {
        get_storage!(boolean::status_key)
    } else {
        false
    };

    require!(
        validate_contract(base58!(dapp)),
        "revoke_confirmation: inv dapp"
    );
    require!(
        validate_contract(base58!(tx_id)),
        "revoke_confirmation: inv tx_id"
    );
    require!(
        contains!(owners, sender_public_key),
        "revoke_confirmation: inv caller"
    );
    require!(
        contains!(confirmations, sender_public_key),
        "revoke_confirmation: not confirmed by caller"
    );
    require!(!status, "revoke_confirmation: already confirmed");

    let mut confirmations_mut = confirmations.as_bytes();
    let mut confirmations_size = 0;
    loop {
        let index = index_of!(confirmations_mut, SEP);
        if index == -1 {
            confirmations_size += 1;
            break;
        }

        confirmations_mut = drop!(confirmations_mut, index + SEP_SIZE);
        confirmations_size += 1;
    }

    confirmations_mut = confirmations.as_bytes();
    let index = index_of!(confirmations_mut, sender_public_key);
    if index == 0 {
        if confirmations_mut.len() > sender_public_key.len() + 2 {
            confirmations_mut = drop!(confirmations_mut, index + SEP_SIZE);
        } else {
            confirmations_mut = drop!(confirmations_mut, index);
        }
    } else {
        let take_from_right =
            confirmations_mut.len() as i64 - index - sender_public_key.len() as i64;
        confirmations_mut = join!(
            binary::take!(confirmations_mut, index - SEP_SIZE),
            take_right!(confirmations_mut, take_from_right)
        );
    }

    set_storage!(string::confirmations_key => confirmations_mut);
}
