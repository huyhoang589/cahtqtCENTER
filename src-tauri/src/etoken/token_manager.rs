use cryptoki::context::{CInitializeArgs, Pkcs11};
use cryptoki::slot::Slot;

use crate::etoken::models::{SlotInfo, TokenInfo};

/// Load the PKCS#11 library from the given path and initialize the context.
pub fn initialize(lib_path: &str) -> Result<Pkcs11, String> {
    let pkcs11 = Pkcs11::new(lib_path)
        .map_err(|e| format!("Failed to load PKCS#11 library '{}': {}", lib_path, e))?;
    pkcs11
        .initialize(CInitializeArgs::OsThreads)
        .map_err(|e| format!("PKCS#11 initialization failed: {}", e))?;
    Ok(pkcs11)
}

/// Enumerate slots with a token present.
/// Returns parallel (slot_infos, raw_slots): raw_slots[i] maps to slot_infos[i].
/// slot_id in SlotInfo is the 0-based index (avoids Slot↔u64 conversion issues in cryptoki 0.6).
pub fn get_all_slots(pkcs11: &Pkcs11) -> Result<(Vec<SlotInfo>, Vec<Slot>), String> {
    let raw_slots = pkcs11
        .get_slots_with_token()
        .map_err(|e| format!("Failed to enumerate token slots: {}", e))?;

    let mut slot_infos = Vec::new();
    for (idx, &slot) in raw_slots.iter().enumerate() {
        let ck_info = pkcs11
            .get_slot_info(slot)
            .map_err(|e| format!("Failed to get slot info: {}", e))?;

        slot_infos.push(SlotInfo {
            slot_id: idx as u64, // 0-based index — avoids Slot↔u64 conversion
            slot_description: ck_info.slot_description().trim().to_string(),
            manufacturer: ck_info.manufacturer_id().trim().to_string(),
            hardware_version: {
                let v = ck_info.hardware_version();
                format!("{}.{}", v.major(), v.minor())
            },
            firmware_version: {
                let v = ck_info.firmware_version();
                format!("{}.{}", v.major(), v.minor())
            },
            token_present: true, // only slots-with-token are returned by get_slots_with_token
        });
    }

    Ok((slot_infos, raw_slots))
}

/// Get token info for each slot. raw_slots must match slot_infos by index.
pub fn get_token_infos(pkcs11: &Pkcs11, slots: &[SlotInfo], raw_slots: &[Slot]) -> Vec<TokenInfo> {
    slots
        .iter()
        .enumerate()
        .filter(|(_, s)| s.token_present)
        .filter_map(|(i, slot_info)| {
            let slot = raw_slots[i];
            let ck = pkcs11.get_token_info(slot).ok()?;
            let fw = ck.firmware_version();
            Some(TokenInfo {
                slot_id: slot_info.slot_id,
                label: ck.label().trim().to_string(),
                manufacturer: ck.manufacturer_id().trim().to_string(),
                model: ck.model().trim().to_string(),
                serial_number: ck.serial_number().trim().to_string(),
                firmware_version: format!("{}.{}", fw.major(), fw.minor()),
                pin_min_len: ck.min_pin_length() as u64,
                pin_max_len: ck.max_pin_length() as u64,
                pin_initialized: ck.token_initialized(),
                user_pin_locked: ck.user_pin_locked(),
                user_pin_final_try: ck.user_pin_final_try(),
                user_pin_count_low: ck.user_pin_count_low(),
            })
        })
        .collect()
}

