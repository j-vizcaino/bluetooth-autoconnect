//! Scans for and monitors Bluetooth devices.

use bluer::{Adapter, AdapterEvent, Address, Device, Error, ErrorKind, Result};
use futures::{pin_mut, StreamExt};
use tokio::time;

struct AutoDevice {
    device: Device,
    name: String,
}

impl AutoDevice {
    pub async fn new(adapter: &Adapter, addr: Address) -> Result<AutoDevice> {
        let device = adapter.device(addr)?;

        if !device.is_trusted().await? {
            return Err(Error {
                kind: ErrorKind::NotAuthorized,
                message: format!("device {} is not trusted", addr),
            });
        }

        let name = device_name(&device).await;
        Ok(AutoDevice { device, name })
    }

    pub async fn auto_connect(self) {
        let sleep_duration = time::Duration::from_secs(10);
        loop {
            self.connect().await;
            time::sleep(sleep_duration).await
        }
    }

    async fn connect(&self) {
        if self.device.is_connected().await.unwrap_or(false) {
            // println!("{}: device is already connected.", self.name);
            return;
        }

        println!("{}: attempting to connect...", self.name,);

        match self.device.connect().await {
            Ok(()) => println!("{}: device connected successfully", self.name),
            Err(e) => println!("{}: connection failed (err={})", self.name, e),
        };
    }
}

async fn device_name(dev: &Device) -> String {
    let alias = dev.alias().await.unwrap_or(String::new());
    if !alias.is_empty() {
        return format!("[{}] {}", dev.address(), alias);
    }

    let name = dev.name().await.unwrap_or(None);
    if name.is_some() {
        return format!("[{}] {}", dev.address(), name.unwrap());
    }

    format!("[{}]", dev.address())
}

#[tokio::main(flavor = "current_thread")]
async fn main() -> Result<()> {
    let session = bluer::Session::new().await?;
    let adapter = session.default_adapter().await?;

    adapter.set_powered(true).await?;

    println!(
        "Discovering on Bluetooth adapter {} with address {}",
        adapter.name(),
        adapter.address().await?
    );

    let device_events = adapter.discover_devices_with_changes().await?;
    pin_mut!(device_events);

    while let Some(device_event) = device_events.next().await {
        match device_event {
            AdapterEvent::DeviceAdded(addr) => {
                match AutoDevice::new(&adapter, addr).await {
                    Ok(auto_dev) => {
                        tokio::spawn(async move {
                            auto_dev.auto_connect().await;
                        });
                    }
                    Err(err) => {
                        println!("{}", err);
                    }
                };
            }
            AdapterEvent::DeviceRemoved(addr) => println!("{:} removed!", addr),
            _ => (),
        }
    }
    Ok(())
}
