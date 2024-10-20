use objc2::rc::Retained;
use objc2_core_bluetooth::CBUUID;
use objc2_foundation::NSString;
use uuid::Uuid;

pub trait IntoCBUUID {
    fn to_cbuuid(self) -> Retained<CBUUID>;
}

impl IntoCBUUID for Uuid {
    fn to_cbuuid(self) -> Retained<CBUUID> {
        unsafe { CBUUID::UUIDWithString(&NSString::from_str(&self.to_string())) }
    }
}
