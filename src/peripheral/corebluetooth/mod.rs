mod characteristic_flags;
mod core_bluetooth_event;
mod error;
mod ffi;
mod into_cbuuid;
pub mod peripheral_delegate;
mod peripheral_manager;

use crate::{
    gatt::service::Service,
    response_channel::{self, response_error::TokenKind},
    Error,
};
use core_bluetooth_event::CoreBluetoothMessage;
use peripheral_delegate::PeripheralDelegateEvent;
use peripheral_manager::run_corebluetooth_thread;
use tokio::sync::mpsc::channel;
use uuid::Uuid;

pub struct Peripheral {
    sender_result: response_channel::Sender<CoreBluetoothMessage, TokenKind>,
}

impl Peripheral {
    pub async fn new() -> Result<Self, Error> {
        let (sender, mut receiver) = channel::<PeripheralDelegateEvent>(256);
        let sender_result = run_corebluetooth_thread(sender);
        if sender_result.is_err() {
            return Err(sender_result.err().unwrap());
        }

        tokio::spawn(async move {
            while let Some(update) = receiver.recv().await {
                match update {
                    PeripheralDelegateEvent::DidUpdateState { state } => {
                        println!("BleOn: {:?}", state)
                    }
                }
            }
        });

        // Store the sender in the Peripheral struct
        Ok(Peripheral {
            sender_result: sender_result.unwrap(),
        })
    }

    pub async fn register_gatt(&self) -> Result<(), Error> {
        Ok(())
    }

    pub async fn unregister_gatt(&self) -> Result<(), Error> {
        Ok(())
    }

    pub async fn is_powered(&self) -> Result<bool, Error> {
        let result: Option<TokenKind> = self.send_event(CoreBluetoothMessage::IsPowered).await;
        let result = result.unwrap();
        if let TokenKind::Boolean(value) = result {
            return Ok(value);
        }
        return Ok(true);
    }

    pub async fn is_advertising(&self) -> Result<bool, Error> {
        let result = self.send_event(CoreBluetoothMessage::IsAdvertising).await;
        if result.is_none() {
            return Err(Error::from_type(crate::ErrorType::Unknown));
        }
        let result = result.unwrap();
        if let TokenKind::Boolean(value) = result {
            return Ok(value);
        }
        return Ok(true);
    }

    pub async fn start_advertising(self: &Self, name: &str, uuids: &[Uuid]) -> Result<(), Error> {
        let result = self
            .send_event(CoreBluetoothMessage::StartAdvertising {
                name: name.to_string(),
                uuids: uuids.to_vec(),
            })
            .await;
        if result.is_none() {
            return Err(Error::from_type(crate::ErrorType::Unknown));
        }
        return Ok(());
    }

    pub async fn stop_advertising(&self) -> Result<(), Error> {
        let result = self.send_event(CoreBluetoothMessage::StopAdvertising).await;
        if result.is_none() {
            return Err(Error::from_type(crate::ErrorType::Unknown));
        }
        return Ok(());
    }

    pub async fn add_service(&self, service: &Service) -> Result<(), Error> {
        let result = self
            .send_event(CoreBluetoothMessage::AddService(service.clone()))
            .await;
        if result.is_none() {
            return Err(Error::from_type(crate::ErrorType::Unknown));
        }
        return Ok(());
    }

    pub async fn send_event(&self, message: CoreBluetoothMessage) -> Option<TokenKind> {
        let result = self
            .sender_result
            .clone()
            .send_await_automatic(message)
            .await;
        if result.is_err() {
            let err = result.err().unwrap();
            println!("Error sending event: {:?}", err);
            return None;
        }
        return result.unwrap();
    }
}
