#[derive(Debug, Clone)]
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

#[derive(Debug, Clone)]
pub enum AttributePermission {
    Readable,
    Writeable,
    ReadEncryptionRequired,
    WriteEncryptionRequired,
}
