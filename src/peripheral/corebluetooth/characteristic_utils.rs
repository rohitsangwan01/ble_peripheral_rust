use crate::gatt::characteristic::{Characteristic, Secure, Write};
use objc2::{rc::Retained, ClassType};
use objc2_core_bluetooth::{
    CBAttributePermissions, CBCharacteristic, CBCharacteristicProperties, CBMutableCharacteristic,
};
use objc2_foundation::NSData;

use super::mac_extensions::UuidExtension;

pub fn parse_characteristic(characteristic: &Characteristic) -> Retained<CBCharacteristic> {
    unsafe {
        let (properties, permissions) = parse_properties_and_permissions(characteristic);

        let value_data = characteristic
            .value
            .as_ref()
            .map(|value| NSData::from_vec(value.clone()));

        return Retained::into_super(
            CBMutableCharacteristic::initWithType_properties_value_permissions(
                CBMutableCharacteristic::alloc(),
                &characteristic.uuid.to_cbuuid(),
                properties,
                value_data.as_ref().map(|data| data as &NSData),
                permissions,
            ),
        );
    }
}

fn parse_properties_and_permissions(
    characteristic: &Characteristic,
) -> (CBCharacteristicProperties, CBAttributePermissions) {
    let mut properties: CBCharacteristicProperties = CBCharacteristicProperties::empty();
    let mut permissions: CBAttributePermissions = CBAttributePermissions::empty();

    if let Some(secure) = &characteristic.properties.read {
        properties |= CBCharacteristicProperties::CBCharacteristicPropertyRead;
        match secure.0 {
            Secure::Secure(_) => permissions |= CBAttributePermissions::ReadEncryptionRequired,
            Secure::Insecure(_) => permissions |= CBAttributePermissions::Readable,
        };
    }

    if let Some(write) = &characteristic.properties.write {
        match write {
            Write::WithResponse(secure) => {
                properties |= CBCharacteristicProperties::CBCharacteristicPropertyWrite;
                match secure {
                    Secure::Secure(_) => {
                        permissions |= CBAttributePermissions::WriteEncryptionRequired
                    }
                    Secure::Insecure(_) => permissions |= CBAttributePermissions::Writeable,
                };
            }
            Write::WithoutResponse(_) => {
                properties |=
                    CBCharacteristicProperties::CBCharacteristicPropertyWriteWithoutResponse
            }
        };
    }

    if characteristic.properties.notify.is_some() {
        properties |= CBCharacteristicProperties::CBCharacteristicPropertyNotify
    }

    if characteristic.properties.indicate.is_some() {
        properties |= CBCharacteristicProperties::CBCharacteristicPropertyIndicate
    }

    (properties, permissions)
}
