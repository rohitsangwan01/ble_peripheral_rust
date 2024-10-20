use objc2::{declare_class, msg_send_id, mutability, rc::Retained, ClassType, DeclaredClass};
use objc2_core_bluetooth::{
    CBManagerState, CBPeripheralManager, CBPeripheralManagerDelegate, CBService,
};
use objc2_foundation::{NSError, NSObject, NSObjectProtocol};
use std::fmt::Debug;
use tokio::sync::mpsc::{self, Sender};

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

        #[method(peripheralManagerDidStartAdvertising:error:)]
        fn delegate_peripheralmanagerdidstartadvertising_error(&self, _: &CBPeripheralManager,error: Option<&NSError>){
            if let Some(error) = error {
                println!("Advertising error: {error}");
            }
            println!("Event:peripheral start advertising");
        }

        #[method(peripheralManager:didAddService:error:)]
         fn delegate_peripheralmanager_didaddservice_error(&self, _: &CBPeripheralManager,service: &CBService, error: Option<&NSError>){
            if let Some(error) = error {
                println!("AddService error: {error}");
            }
            println!("Event:AddService {:?}",service);
        }
    }
);

impl PeripheralDelegate {
    pub fn new(sender: mpsc::Sender<PeripheralDelegateEvent>) -> Retained<Self> {
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
