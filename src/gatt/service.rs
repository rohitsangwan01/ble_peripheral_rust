use super::characteristic::Characteristic;
use uuid::Uuid;

#[derive(Debug, Clone)]
pub struct Service {
    pub uuid: Uuid,
    pub primary: bool,
    pub characteristics: Vec<Characteristic>,
}

impl Service {
    pub fn new(uuid: Uuid, primary: bool, characteristics: Vec<Characteristic>) -> Self {
        Service {
            uuid,
            primary,
            characteristics,
        }
    }
}
