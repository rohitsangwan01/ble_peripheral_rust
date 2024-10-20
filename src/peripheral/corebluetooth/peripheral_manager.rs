use super::characteristic_utils::parse_characteristic;
use super::mac_extensions::UuidExtension as _;
use super::peripheral_delegate::PeripheralDelegate;
use crate::gatt::peripheral_event::PeripheralEvent;
use crate::gatt::service::Service;
use crate::Error;
use objc2::{rc::Retained, runtime::AnyObject, ClassType};
use objc2_core_bluetooth::{
    CBAdvertisementDataLocalNameKey, CBAdvertisementDataServiceUUIDsKey, CBCharacteristic,
    CBManager, CBManagerAuthorization, CBManagerState, CBMutableService, CBPeripheralManager,
};
use objc2_foundation::{NSArray, NSDictionary, NSString};
use std::sync::Arc;
use tokio::sync::mpsc;
use uuid::Uuid;

#[derive(Debug)]
pub struct PeripheralManager {
    cb_peripheral_manager: Retained<CBPeripheralManager>,
    #[allow(dead_code)] // Keep peripheral_delegate to maintain delegate lifecycle
    peripheral_delegate: Arc<Retained<PeripheralDelegate>>,
}

impl PeripheralManager {
    pub fn new(sender_tx: mpsc::Sender<PeripheralEvent>) -> Result<Self, Error> {
        if !is_authorized() {
            return Err(Error::from_type(crate::ErrorType::PermissionDenied));
        }

        let result: (
            Retained<CBPeripheralManager>,
            Arc<Retained<PeripheralDelegate>>,
        ) = PeripheralDelegate::new(sender_tx);

        Ok(Self {
            cb_peripheral_manager: result.0,
            peripheral_delegate: result.1,
        })
    }

    pub fn is_powered(self: &Self) -> bool {
        unsafe {
            let state = self.cb_peripheral_manager.state();
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
            self.cb_peripheral_manager
                .startAdvertising(Some(&advertising_data));
        }
    }

    pub fn stop_advertising(self: &Self) {
        unsafe {
            self.cb_peripheral_manager.stopAdvertising();
        }
    }

    pub fn is_advertising(self: &Self) -> bool {
        unsafe { self.cb_peripheral_manager.isAdvertising() }
    }

    // Peripheral with cache value must only have Read permission, else it will crash
    // TODO: throw proper error, or catch Objc errors
    pub fn add_service(self: &Self, service: &Service) {
        unsafe {
            let characteristics: Vec<Retained<CBCharacteristic>> = service
                .characteristics
                .iter()
                .map(|characteristic| parse_characteristic(characteristic))
                .collect();

            let mutable_service: Retained<CBMutableService> =
                CBMutableService::initWithType_primary(
                    CBMutableService::alloc(),
                    &service.uuid.to_cbuuid(),
                    service.primary,
                );

            if !characteristics.is_empty() {
                let chars = NSArray::from_vec(characteristics);
                mutable_service.setCharacteristics(Some(&chars));
            }

            self.cb_peripheral_manager.addService(&mutable_service);
        }
    }
}

pub fn is_authorized() -> bool {
    let authorization = unsafe { CBManager::authorization_class() };
    return authorization != CBManagerAuthorization::Restricted
        && authorization != CBManagerAuthorization::Denied;
}
