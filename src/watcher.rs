use bluer::{Adapter, AdapterEvent, Address, Device};
use futures::{pin_mut, StreamExt};
use std::collections::HashMap;
use tokio::sync::mpsc;

#[derive(Debug)]
pub enum Event {
    DeviceAdded(Device),
    DeviceRemoved(Device),
}

pub struct TrustedWatcher {
    adapter: Adapter,
    devices: HashMap<Address, Device>,
    events: mpsc::UnboundedSender<Event>,
}

impl TrustedWatcher {
    pub fn new(adapter: &Adapter, events_tx: mpsc::UnboundedSender<Event>) -> TrustedWatcher {
        TrustedWatcher {
            adapter: adapter.clone(),
            devices: HashMap::new(),
            events: events_tx,
        }
    }

    pub async fn start(&mut self) {
        let device_events = self.adapter.discover_devices_with_changes().await.unwrap();
        pin_mut!(device_events);

        while let Some(device_event) = device_events.next().await {
            match device_event {
                AdapterEvent::DeviceAdded(addr) => self.on_device_added(addr).await,
                AdapterEvent::DeviceRemoved(addr) => self.on_device_removed(addr),
                _ => (),
            }
        }
    }

    async fn on_device_added(&mut self, addr: Address) {
        if self.devices.contains_key(&addr) {
            return;
        }

        match self.adapter.device(addr) {
            Ok(dev) => {
                if !dev.is_trusted().await.unwrap_or(false) {
                    return;
                }
                self.watch_device(dev);
            }
            Err(err) => println!("Failed to resolve device from address {}: {}", addr, err),
        }
    }

    fn watch_device(&mut self, dev: Device) {
        self.devices.insert(dev.address(), dev.clone());
        self.events.send(Event::DeviceAdded(dev)).unwrap();
    }

    fn on_device_removed(&mut self, addr: Address) {
        if !self.devices.contains_key(&addr) {
            return;
        }

        match self.devices.remove(&addr) {
            Some(dev) => self.events.send(Event::DeviceRemoved(dev)).unwrap(),
            None => (),
        }
    }
}
