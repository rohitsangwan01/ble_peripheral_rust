#[derive(Debug, Clone, PartialEq)]
pub enum CharacteristicProperty {
    Broadcast,
    Read,
    WriteWithoutResponse,
    Write,
    Notify,
    Indicate,
    AuthenticatedSignedWrites,
    ExtendedProperties,
    NotifyEncryptionRequired,
    IndicateEncryptionRequired,
}

#[derive(Debug, Clone, PartialEq)]
pub enum AttributePermission {
    Readable,
    Writeable,
    ReadEncryptionRequired,
    WriteEncryptionRequired,
}
