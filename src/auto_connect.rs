use crate::helpers::pretty_device_name;
use bluer::Device;
use tokio::time::{sleep, Duration};

pub struct AutoConnect {
    device: Device,
    device_name: String,
    retry_interval: Duration,
}

impl AutoConnect {
    pub async fn new(device: Device, retry_interval: Duration) -> AutoConnect {
        let device_name = pretty_device_name(&device).await;
        AutoConnect {
            device,
            device_name,
            retry_interval,
        }
    }

    pub async fn run(&self) {
        loop {
            self.connect_device().await;
            sleep(self.retry_interval).await;
        }
    }

    async fn connect_device(&self) {
        if self.device.is_connected().await.unwrap_or(false) {
            // println!("{}: device is already connected.", self.name);
            return;
        }

        println!("{}: attempting to connect...", self.device_name,);

        match self.device.connect().await {
            Ok(()) => println!("{}: device connected successfully", self.device_name),
            Err(e) => println!("{}: connection failed (err={})", self.device_name, e),
        };
    }
}
