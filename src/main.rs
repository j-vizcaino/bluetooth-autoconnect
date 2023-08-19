use std::collections::HashMap;
use std::sync::Arc;
use tokio::time::Duration;

mod auto_connect;
mod helpers;
mod watcher;

#[tokio::main(flavor = "current_thread")]
async fn main() {
    let retry_interval = Duration::from_secs(10);

    println!("Initializing Bluetooth adapter...");
    let adapter = helpers::bluetooth_adapter().await;

    println!(
        "Discovering on Bluetooth adapter {} with address {}",
        adapter.name(),
        adapter.address().await.unwrap()
    );
    let (tx_events, mut rx_events) = tokio::sync::mpsc::unbounded_channel::<watcher::Event>();
    let mut dev_watcher = watcher::TrustedWatcher::new(&adapter, tx_events);

    let mut connectors = HashMap::new();

    let watcher_task = dev_watcher.start();
    tokio::pin!(watcher_task);

    loop {
        tokio::select! {
            _ = &mut watcher_task => break,
            Some(evt) = rx_events.recv() => {
                println!("{:?}", evt);
                match evt {
                    watcher::Event::DeviceAdded(dev) => {
                        let dev_address = dev.address();
                        let connector = auto_connect::AutoConnect::new(dev, retry_interval).await;

                        let shared_connector = Arc::new(connector);
                        connectors.insert(dev_address, shared_connector.clone());
                        tokio::task::spawn( async move {
                            shared_connector.run().await
                        });
                    }
                    watcher::Event::DeviceRemoved(dev) => {
                        let dev_address = dev.address();

                        if !connectors.contains_key(&dev_address) {
                            println!("Unknown device received in DeviceRemoved event: {:?}", dev);
                        }
                    }
                }
            }
        }
    }
}
