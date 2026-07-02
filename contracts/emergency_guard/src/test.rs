#![cfg(test)]

use soroban_sdk::{testutils::Address as _, vec, Address, Env};

#[test]
fn test_emergency_guard_initialization() {
    let env = Env::default();
    let admin1 = Address::generate(&env);
    let admin2 = Address::generate(&env);
    let admins = vec![&env, admin1.clone(), admin2.clone()];

    // This would be called during contract initialization
    // For testing, we're just verifying the PauseType structure works
    let pause_state = crate::PauseType::new(0);
    assert_eq!(pause_state.0, 0);
}

#[test]
fn test_granular_pause_types() {
    let mut pause = crate::PauseType::new(0);

    // Test SWAP pause
    pause.set_paused(crate::PauseType::SWAP, true);
    assert!(pause.is_paused(crate::PauseType::SWAP));
    assert!(!pause.is_paused(crate::PauseType::DEPOSIT));

    // Test DEPOSIT pause
    pause.set_paused(crate::PauseType::DEPOSIT, true);
    assert!(pause.is_paused(crate::PauseType::SWAP));
    assert!(pause.is_paused(crate::PauseType::DEPOSIT));

    // Test WITHDRAW pause
    pause.set_paused(crate::PauseType::WITHDRAW, true);
    assert!(pause.is_paused(crate::PauseType::SWAP));
    assert!(pause.is_paused(crate::PauseType::DEPOSIT));
    assert!(pause.is_paused(crate::PauseType::WITHDRAW));

    // Test unpausing
    pause.set_paused(crate::PauseType::SWAP, false);
    assert!(!pause.is_paused(crate::PauseType::SWAP));
    assert!(pause.is_paused(crate::PauseType::DEPOSIT));
    assert!(pause.is_paused(crate::PauseType::WITHDRAW));
}

#[test]
fn test_pause_all_and_unpause_all() {
    let mut pause = crate::PauseType::new(0);

    // Pause all
    pause.pause_all();
    assert!(pause.is_paused(crate::PauseType::SWAP));
    assert!(pause.is_paused(crate::PauseType::DEPOSIT));
    assert!(pause.is_paused(crate::PauseType::WITHDRAW));
    assert!(pause.is_paused(crate::PauseType::TRANSFER));
    assert!(pause.is_paused(crate::PauseType::MINT));
    assert!(pause.is_paused(crate::PauseType::BURN));

    // Unpause all
    pause.unpause_all();
    assert!(!pause.is_paused(crate::PauseType::SWAP));
    assert!(!pause.is_paused(crate::PauseType::DEPOSIT));
    assert!(!pause.is_paused(crate::PauseType::WITHDRAW));
    assert!(!pause.is_paused(crate::PauseType::TRANSFER));
    assert!(!pause.is_paused(crate::PauseType::MINT));
    assert!(!pause.is_paused(crate::PauseType::BURN));
}

#[test]
fn test_multiple_pause_types() {
    let mut pause = crate::PauseType::new(0);

    // Create a custom pause state with multiple operations
    let combined = crate::PauseType::SWAP | crate::PauseType::DEPOSIT | crate::PauseType::MINT;
    pause.set_paused(combined, true);

    assert!(pause.is_paused(crate::PauseType::SWAP));
    assert!(pause.is_paused(crate::PauseType::DEPOSIT));
    assert!(!pause.is_paused(crate::PauseType::WITHDRAW));
    assert!(pause.is_paused(crate::PauseType::MINT));
    assert!(!pause.is_paused(crate::PauseType::BURN));
}

#[test]
fn test_bitmask_storage_benchmark_reports_lower_footprint_than_mapping() {
    let env = Env::default();
    let contract_id = env.register(crate::EmergencyGuard, ());
    let client = crate::EmergencyGuardClient::new(&env, &contract_id);

    let stats = client.benchmark_storage(&6);

    assert!(stats.bitmask_storage_entries < stats.mapping_storage_entries);
    assert_eq!(stats.bitmask_storage_entries, 1);
    assert_eq!(stats.mapping_storage_entries, 6);
    assert!(stats.bitmask_storage_reads <= stats.mapping_storage_reads);
    assert!(stats.bitmask_storage_writes <= stats.mapping_storage_writes);
}

#[test]
fn test_guard_can_initialize_and_pause_operations() {
    let e = Env::default();
    e.mock_all_auths();

    let contract_id = e.register(crate::EmergencyGuard, ());
    let client = crate::EmergencyGuardClient::new(&e, &contract_id);

    let admin = Address::generate(&e);
    let admins = vec![&e, admin.clone()];

    client.initialize(&admins, &1u32);
    client.set_pause(&admin, &crate::PauseType::TRANSFER, &true);

    assert!(client.is_paused(&crate::PauseType::TRANSFER));
}
