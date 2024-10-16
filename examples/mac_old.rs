use ble_peripheral_rust::macos_old::peripheral_delegate::{
    PeripheralDelegate, PeripheralDelegateEvent,
};
use ble_peripheral_rust::macos_old::peripheral_manager::{check_permission, PeripheralManager};
use objc2_core_bluetooth::CBManagerState;
use tokio::sync::mpsc::channel;
use std::sync::Arc;

#[tokio::main]
async fn main() {
    if !check_permission() {
        println!("Bluetooth Permission not granted")
    }

    let (sender, mut receiver) = channel::<PeripheralDelegateEvent>(256);

    let peripheral_delegate = Arc::new(PeripheralDelegate::new(sender));
    let peripheral_manager = match PeripheralManager::new(peripheral_delegate.clone()) {
        Ok(peripheral_manager) => peripheral_manager,
        Err(e) => {
            log::error!("Error: {}", e);
            return;
        }
    };

    let is_advertising = peripheral_manager.is_advertising();
    println!("Isadvertising: {}", is_advertising);

    while let Some(update) = receiver.recv().await {
        handle_update(update);
    }
}

fn handle_update(update: PeripheralDelegateEvent) {
    match update {
        PeripheralDelegateEvent::DidUpdateState { state } => {
            println!("BleOn: {}", state == CBManagerState::PoweredOn)
        }
    }
}
