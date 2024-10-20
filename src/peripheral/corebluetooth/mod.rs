mod characteristic_utils;
mod mac_extensions;
mod mac_utils;
pub mod peripheral_delegate;
mod peripheral_manager;

use crate::{gatt::service::Service, Error};
use peripheral_delegate::PeripheralDelegateEvent;
use peripheral_manager::PeripheralManager;
use tokio::sync::mpsc;
use uuid::Uuid;

pub struct Peripheral {
    peripheral_manager: PeripheralManager,
}

impl Peripheral {
    pub async fn new() -> Result<Self, Error> {
        let (sender_tx, mut sender_rx) = mpsc::channel::<PeripheralDelegateEvent>(256);
        let peripheral_manager = PeripheralManager::new(sender_tx).unwrap();
        tokio::spawn(async move {
            while let Some(update) = sender_rx.recv().await {
                handle_updates(update);
            }
        });
        Ok(Peripheral { peripheral_manager })
    }

    pub async fn register_gatt(&self) -> Result<(), Error> {
        Ok(())
    }

    pub async fn unregister_gatt(&self) -> Result<(), Error> {
        Ok(())
    }

    pub async fn is_powered(&self) -> Result<bool, Error> {
        return Ok(self.peripheral_manager.is_powered());
    }

    pub async fn is_advertising(&self) -> Result<bool, Error> {
        return Ok(self.peripheral_manager.is_advertising());
    }

    pub async fn start_advertising(self: &Self, name: &str, uuids: &[Uuid]) -> Result<(), Error> {
        return Ok(self.peripheral_manager.start_advertising(name, uuids));
    }

    pub async fn stop_advertising(&self) -> Result<(), Error> {
        return Ok(self.peripheral_manager.stop_advertising());
    }

    pub async fn add_service(&self, service: &Service) -> Result<(), Error> {
        return Ok(self.peripheral_manager.add_service(service));
    }
}

pub fn handle_updates(update: PeripheralDelegateEvent) {
    match update {
        PeripheralDelegateEvent::DidUpdateState { state } => {
            println!("BleOn: {:?}", state)
        }
        PeripheralDelegateEvent::DidStartAdverising { error } => {
            println!("DidStartAdvertising: {:?}", error)
        }
        PeripheralDelegateEvent::DidAddService { service, error } => {
            println!("DidAddService: {:?} {:?}", service, error)
        }
        PeripheralDelegateEvent::DidSubscribeToCharacteristic {
            client,
            service,
            characteristic,
        } => {
            println!(
                "DidSubscribeToCharacteristic: {:?} {:?} {:?}",
                client, service, characteristic
            )
        }
        PeripheralDelegateEvent::DidUnsubscribeFromCharacteristic {
            client,
            service,
            characteristic,
        } => {
            println!(
                "DidUnsubscribeFromCharacteristic: {:?} {:?} {:?}",
                client, service, characteristic
            )
        }
        PeripheralDelegateEvent::DidReceiveReadRequest {
            client,
            service,
            characteristic,
        } => {
            println!(
                "DidReceiveReadRequest: {:?} {:?} {:?}",
                client, service, characteristic
            )
        }
        PeripheralDelegateEvent::DidReceiveWriteRequest {
            client,
            service,
            characteristic,
        } => {
            println!(
                "DidReceiveWriteRequest: {:?} {:?} {:?}",
                client, service, characteristic
            )
        }
    }
}
