use objc2::runtime::{AnyObject, ProtocolObject};
use objc2::{declare_class, msg_send_id, mutability, rc::Retained, ClassType, DeclaredClass};
use objc2_core_bluetooth::{
    CBAdvertisementDataLocalNameKey, CBAdvertisementDataManufacturerDataKey,
    CBAdvertisementDataServiceDataKey, CBAdvertisementDataServiceUUIDsKey, CBCentralManager,
    CBCentralManagerDelegate, CBCharacteristic, CBDescriptor, CBManagerState, CBPeripheral,
    CBPeripheralDelegate, CBPeripheralManager, CBPeripheralManagerDelegate, CBService, CBUUID,
};
use objc2_foundation::{
    NSArray, NSData, NSDictionary, NSError, NSNumber, NSObject, NSObjectProtocol, NSString,
};
use std::{
    collections::HashMap,
    fmt::{self, Debug, Formatter},
    ops::Deref,
};
use tokio::sync::mpsc::Sender;

pub enum PeripheralDelegateEvent {
    DidUpdateState { state: CBManagerState },
}

declare_class!(
    #[derive(Debug)]
    pub struct PeripheralDelegate;

    unsafe impl ClassType for PeripheralDelegate {
        type Super = NSObject;
        type Mutability = mutability::InteriorMutable;
        const NAME: &'static str = "PeripheralManagerDelegate";
    }

    impl DeclaredClass for PeripheralDelegate {
        type Ivars = Sender<PeripheralDelegateEvent>;
    }

    unsafe impl NSObjectProtocol for PeripheralDelegate {}

    unsafe impl CBPeripheralManagerDelegate for PeripheralDelegate {
        #[method(peripheralManagerDidUpdateState:)]
         fn delegate_peripheralmanagerdidupdatestate(&self, peripheral: &CBPeripheralManager){
                let state = unsafe { peripheral.state() };
                self.send_event(PeripheralDelegateEvent::DidUpdateState { state });
         }
    }
);

impl PeripheralDelegate {
    pub fn new(sender: Sender<PeripheralDelegateEvent>) -> Retained<Self> {
        let this = PeripheralDelegate::alloc().set_ivars(sender);
        unsafe { msg_send_id![super(this), init] }
    }

    fn send_event(&self, event: PeripheralDelegateEvent) {
        let sender = self.ivars().clone();
        futures::executor::block_on(async {
            if let Err(e) = sender.send(event).await {
                println!("Error sending delegate event: {}", e);
            }
        });
    }
}
