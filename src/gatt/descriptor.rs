use super::properties::{AttributePermission, CharacteristicProperty};
use uuid::Uuid;

#[derive(Debug, Clone)]
pub struct Descriptor {
    pub uuid: Uuid,
    pub properties: Vec<CharacteristicProperty>,
    pub permissions: Vec<AttributePermission>,
    pub value: Option<Vec<u8>>,
}

impl Descriptor {
    pub fn new(
        uuid: Uuid,
        properties: Vec<CharacteristicProperty>,
        permissions: Vec<AttributePermission>,
        value: Option<Vec<u8>>,
    ) -> Self {
        Descriptor {
            uuid,
            properties,
            permissions,
            value,
        }
    }
}
