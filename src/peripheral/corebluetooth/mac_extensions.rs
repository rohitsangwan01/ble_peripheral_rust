use objc2::rc::Retained;
use objc2_core_bluetooth::{CBCharacteristic, CBService, CBUUID};
use objc2_foundation::NSString;
use uuid::Uuid;

pub trait UuidExtension {
    fn to_cbuuid(self) -> Retained<CBUUID>;
}

impl UuidExtension for Uuid {
    fn to_cbuuid(self) -> Retained<CBUUID> {
        unsafe { CBUUID::UUIDWithString(&NSString::from_str(&self.to_string())) }
    }
}

pub trait UuidHelper {
    fn get_uuid(self) -> Uuid;
}

impl UuidHelper for &CBService {
    fn get_uuid(self) -> Uuid {
        unsafe {
            return self.UUID().as_ref().to_uuid();
        }
    }
}

impl UuidHelper for &CBCharacteristic {
    fn get_uuid(self) -> Uuid {
        unsafe {
            return self.UUID().as_ref().to_uuid();
        }
    }
}

pub trait CbUuidExtension {
    fn to_uuid(self) -> Uuid;
}

impl CbUuidExtension for &CBUUID {
    fn to_uuid(self) -> Uuid {
        // NOTE: CoreBluetooth tends to return uppercase UUID strings, and only 4
        // character long if the UUID is short (16 bits). It can also return 8
        // character strings if the rest of the UUID matches the generic UUID.
        let uuid = unsafe { self.UUIDString() }.to_string();
        let long = if uuid.len() == 4 {
            format!("0000{}-0000-1000-8000-00805f9b34fb", uuid)
        } else if uuid.len() == 8 {
            format!("{}-0000-1000-8000-00805f9b34fb", uuid)
        } else {
            uuid
        };
        let uuid_string = long.to_lowercase();
        uuid_string.parse().unwrap()
    }
}
