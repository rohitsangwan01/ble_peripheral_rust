use uuid::Uuid;

use crate::gatt::service::Service;

pub enum CoreBluetoothMessage {
    StartAdvertising { name: String, uuids: Vec<Uuid> },
    StopAdvertising,
    AddService(Service),
    IsPowered,
    IsAdvertising,
}
