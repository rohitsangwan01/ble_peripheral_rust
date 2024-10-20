use super::core_bluetooth_event::CoreBluetoothMessage;
use super::peripheral_delegate::PeripheralDelegateEvent;
use super::{characteristic_flags::get_properties_and_permissions, into_cbuuid::IntoCBUUID};
use super::{ffi, peripheral_delegate::PeripheralDelegate};
use crate::gatt::service::Service;
use crate::response_channel::response_error::TokenKind;
use crate::{response_channel, Error};
use log::{trace, warn};
use objc2::{msg_send_id, rc::Retained, runtime::AnyObject, ClassType};
use objc2_core_bluetooth::{
    CBAdvertisementDataLocalNameKey, CBAdvertisementDataServiceUUIDsKey, CBAttributePermissions,
    CBCharacteristic, CBCharacteristicProperties, CBManager, CBManagerAuthorization,
    CBManagerState, CBMutableCharacteristic, CBMutableService, CBPeripheralManager,
};
use objc2_foundation::{NSArray, NSData, NSDictionary, NSString};
use std::ffi::CString;
use std::sync::Arc;
use std::thread;
use tokio::runtime;
use tokio::sync::mpsc::{Receiver, Sender};
use uuid::Uuid;

#[derive(Debug)]
pub struct PeripheralManager {
    peripheral_manager_delegate: Retained<CBPeripheralManager>,
    receiver: Receiver<(CoreBluetoothMessage, Sender<TokenKind>)>,
}

impl PeripheralManager {
    pub fn new(
        peripheral_delegate: Arc<Retained<PeripheralDelegate>>,
        receiver: Receiver<(CoreBluetoothMessage, Sender<TokenKind>)>,
    ) -> Result<Self, Error> {
        let authorization = unsafe { CBManager::authorization_class() };
        if authorization != CBManagerAuthorization::AllowedAlways
            && authorization != CBManagerAuthorization::NotDetermined
        {
            warn!("Authorization status {:?}", authorization);
            return Err(Error::from_type(crate::ErrorType::PermissionDenied));
        } else {
            trace!("Authorization status {:?}", authorization);
        }

        let label: CString = CString::new("CBqueue").unwrap();
        let queue: *mut std::ffi::c_void =
            unsafe { ffi::dispatch_queue_create(label.as_ptr(), ffi::DISPATCH_QUEUE_SERIAL) };
        let queue: *mut AnyObject = queue.cast();
        let peripheral_manager_delegate: Retained<CBPeripheralManager> = unsafe {
            msg_send_id![CBPeripheralManager::alloc(), initWithDelegate: &**peripheral_delegate, queue: queue]
        };
        Ok(Self {
            peripheral_manager_delegate,
            receiver: receiver,
        })
    }

    pub async fn wait_for_message(&mut self) {
        if let Some((message, tx)) = self.receiver.recv().await {
            let token_kind: TokenKind = match message {
                CoreBluetoothMessage::StartAdvertising { name, uuids } => {
                    TokenKind::Ok(self.start_advertising(&name, &uuids))
                }
                CoreBluetoothMessage::StopAdvertising => TokenKind::Ok(self.stop_advertising()),
                CoreBluetoothMessage::AddService(service) => {
                    TokenKind::Ok(self.add_service(&service))
                }
                CoreBluetoothMessage::IsPowered => TokenKind::Boolean(self.is_powered()),
                CoreBluetoothMessage::IsAdvertising => TokenKind::Boolean(self.is_advertising()),
            };
            let result = tx.send(token_kind).await;
            if let Err(result) = result {
                println!("Error sending tokenKind: {:?}", result);
            }
        }
    }

    pub fn is_powered(self: &Self) -> bool {
        unsafe {
            let state = self.peripheral_manager_delegate.state();
            println!("State: {:?}", state);
            state == CBManagerState::PoweredOn
        }
    }

    pub fn start_advertising(self: &Self, name: &str, uuids: &[Uuid]) {
        let mut keys: Vec<&NSString> = vec![];
        let mut objects: Vec<Retained<AnyObject>> = vec![];

        unsafe {
            keys.push(CBAdvertisementDataLocalNameKey);
            objects.push(Retained::cast(NSString::from_str(name)));

            keys.push(CBAdvertisementDataServiceUUIDsKey);
            objects.push(Retained::cast(NSArray::from_vec(
                uuids.iter().map(|u| u.to_cbuuid()).collect(),
            )));
        }

        let advertising_data: Retained<NSDictionary<NSString, AnyObject>> =
            NSDictionary::from_vec(&keys, objects);

        unsafe {
            println!("Starting advetisemet");
            self.peripheral_manager_delegate
                .startAdvertising(Some(&advertising_data));
        }
    }

    pub fn stop_advertising(self: &Self) {
        unsafe {
            self.peripheral_manager_delegate.stopAdvertising();
        }
    }

    pub fn is_advertising(self: &Self) -> bool {
        unsafe { self.peripheral_manager_delegate.isAdvertising() }
    }

    pub fn add_service(self: &Self, service: &Service) {
        let characteristics: Vec<Retained<CBCharacteristic>> = service
            .characteristics
            .iter()
            .map(|characteristic| {
                let (properties, permissions) = get_properties_and_permissions(characteristic);
                unsafe {
                    let mutable_char = match characteristic.value.clone() {
                        Some(value) => {
                            CBMutableCharacteristic::initWithType_properties_value_permissions(
                                CBMutableCharacteristic::alloc(),
                                &characteristic.uuid.to_cbuuid(),
                                CBCharacteristicProperties::from_bits(properties as usize).unwrap(),
                                Some(&NSData::from_vec(value.clone())),
                                CBAttributePermissions::from_bits(permissions as usize).unwrap(),
                            )
                        }
                        None => CBMutableCharacteristic::initWithType_properties_value_permissions(
                            CBMutableCharacteristic::alloc(),
                            &characteristic.uuid.to_cbuuid(),
                            CBCharacteristicProperties::from_bits(properties as usize).unwrap(),
                            None,
                            CBAttributePermissions::from_bits(permissions as usize).unwrap(),
                        ),
                    };
                    return Retained::into_super(mutable_char);
                }
            })
            .collect();

        unsafe {
            let mutable_service = CBMutableService::initWithType_primary(
                CBMutableService::alloc(),
                &service.uuid.to_cbuuid(),
                service.primary,
            );

            if !characteristics.is_empty() {
                let chars = NSArray::from_vec(characteristics);
                mutable_service.setCharacteristics(Some(&chars));
            }

            self.peripheral_manager_delegate
                .addService(&mutable_service);
            println!("Added Services")
        }
    }
}

pub fn run_corebluetooth_thread(
    event_sender: Sender<PeripheralDelegateEvent>,
) -> Result<response_channel::Sender<CoreBluetoothMessage, TokenKind>, Error> {
    let (sender, receiver) =
        response_channel::channel::<CoreBluetoothMessage, TokenKind>(256, None);

    thread::spawn(move || {
        let runtime = runtime::Builder::new_current_thread().build().unwrap();
        runtime.block_on(async move {
            println!("Runtime Started");
            let peripheral_delegate = Arc::new(PeripheralDelegate::new(event_sender));
            let mut peripheral_manager =
                PeripheralManager::new(peripheral_delegate.clone(), receiver).unwrap();
            loop {
                peripheral_manager.wait_for_message().await;
            }
        })
    });

    Ok(sender)
}
