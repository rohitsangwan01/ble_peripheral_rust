use uuid::Uuid;

#[derive(Debug, Clone)]
pub enum PeripheralEvent {
    DidUpdateState {
        is_powered: bool,
    },
    DidStartAdverising {
        error: Option<String>,
    },
    DidAddService {
        service: Uuid,
        error: Option<String>,
    },
    DidSubscribeToCharacteristic {
        client: String,
        service: Uuid,
        characteristic: Uuid,
    },
    DidUnsubscribeFromCharacteristic {
        client: String,
        service: Uuid,
        characteristic: Uuid,
    },
    DidReceiveReadRequest {
        client: String,
        service: Uuid,
        characteristic: Uuid,
    },
    DidReceiveWriteRequest {
        client: String,
        service: Uuid,
        characteristic: Uuid,
        value: Vec<u8>,
    },
}
