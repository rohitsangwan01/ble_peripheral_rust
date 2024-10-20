use super::{
    descriptor::Descriptor,
    properties::{AttributePermission, CharacteristicProperty},
};
use uuid::Uuid;

#[derive(Debug, Clone)]
pub struct Characteristic {
    pub uuid: Uuid,
    pub properties: Vec<CharacteristicProperty>,
    pub permissions: Vec<AttributePermission>,
    pub value: Option<Vec<u8>>,
    pub descriptors: Vec<Descriptor>,
}

impl Characteristic {
    pub fn new(
        uuid: Uuid,
        properties: Vec<CharacteristicProperty>,
        permissions: Vec<AttributePermission>,
        value: Option<Vec<u8>>,
        descriptors: Vec<Descriptor>,
    ) -> Self {
        Characteristic {
            uuid,
            properties,
            permissions,
            value,
            descriptors,
        }
    }
}
