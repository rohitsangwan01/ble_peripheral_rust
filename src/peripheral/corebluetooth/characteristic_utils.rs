use crate::gatt::{
    characteristic::Characteristic,
    properties::{AttributePermission, CharacteristicProperty},
};
use objc2::{rc::Retained, ClassType};
use objc2_core_bluetooth::{
    CBAttributePermissions, CBCharacteristic, CBCharacteristicProperties, CBMutableCharacteristic,
};
use objc2_foundation::NSData;

use super::mac_extensions::UuidExtension;

pub fn parse_characteristic(characteristic: &Characteristic) -> Retained<CBCharacteristic> {
    unsafe {
        let properties = characteristic
            .properties
            .iter()
            .fold(CBCharacteristicProperties::empty(), |acc, property| {
                acc | property.clone().to_cb_property()
            });

        println!("Properties: {:?}", properties);

        let permissions = characteristic
            .permissions
            .iter()
            .fold(CBAttributePermissions::empty(), |acc, permission| {
                acc | permission.clone().to_attribute_permission()
            });
        println!("permissions: {:?}", permissions);

        let value_data = characteristic
            .value
            .as_ref()
            .map(|value| NSData::from_vec(value.clone()));

        let mutable_char = CBMutableCharacteristic::initWithType_properties_value_permissions(
            CBMutableCharacteristic::alloc(),
            &characteristic.uuid.to_cbuuid(),
            properties,
            value_data.as_ref().map(|data| data as &NSData),
            permissions,
        );

        // let descriptors: Retained<NSArray<CBDescriptor>> = NSArray::from_vec(
        //     characteristic
        //         .descriptors
        //         .iter()
        //         .map(|desc| parse_descriptor(desc))
        //         .collect(),
        // );

        // mutable_char.setDescriptors(Some(&descriptors));

        return Retained::into_super(mutable_char);
    }
}

// pub fn parse_descriptor(descriptor: &Descriptor) -> Retained<CBDescriptor> {
//     unsafe {
//         let value_data = descriptor
//             .value
//             .as_ref()
//             .map(|value| NSData::from_vec(value.clone()));

//         return Retained::into_super(CBMutableDescriptor::initWithType_value(
//             CBMutableDescriptor::alloc(),
//             &descriptor.uuid.to_cbuuid(),
//             value_data.as_ref().map(|data| data as &AnyObject),
//         ));
//     }
// }

impl CharacteristicProperty {
    fn to_cb_property(self) -> CBCharacteristicProperties {
        return match self {
            CharacteristicProperty::Broadcast => {
                CBCharacteristicProperties::CBCharacteristicPropertyBroadcast
            }
            CharacteristicProperty::Read => {
                CBCharacteristicProperties::CBCharacteristicPropertyRead
            }
            CharacteristicProperty::WriteWithoutResponse => {
                CBCharacteristicProperties::CBCharacteristicPropertyWriteWithoutResponse
            }
            CharacteristicProperty::Write => {
                CBCharacteristicProperties::CBCharacteristicPropertyWrite
            }
            CharacteristicProperty::Notify => {
                CBCharacteristicProperties::CBCharacteristicPropertyNotify
            }
            CharacteristicProperty::NotifyEncryptionRequired => {
                CBCharacteristicProperties::CBCharacteristicPropertyNotifyEncryptionRequired
            }
            CharacteristicProperty::Indicate => {
                CBCharacteristicProperties::CBCharacteristicPropertyIndicate
            }
            CharacteristicProperty::IndicateEncryptionRequired => {
                CBCharacteristicProperties::CBCharacteristicPropertyIndicateEncryptionRequired
            }
            CharacteristicProperty::AuthenticatedSignedWrites => {
                CBCharacteristicProperties::CBCharacteristicPropertyAuthenticatedSignedWrites
            }
            CharacteristicProperty::ExtendedProperties => {
                CBCharacteristicProperties::CBCharacteristicPropertyExtendedProperties
            }
        };
    }
}

impl AttributePermission {
    fn to_attribute_permission(self) -> CBAttributePermissions {
        return match self {
            AttributePermission::Readable => CBAttributePermissions::Readable,
            AttributePermission::Writeable => CBAttributePermissions::Writeable,
            AttributePermission::ReadEncryptionRequired => {
                CBAttributePermissions::ReadEncryptionRequired
            }
            AttributePermission::WriteEncryptionRequired => {
                CBAttributePermissions::WriteEncryptionRequired
            }
        };
    }
}
