use backon::{ConstantBuilder, Retryable};
use bluer::{Adapter, Device, Result};
use std::time::Duration;

async fn try_bluetooth_adapter() -> Result<Adapter> {
    let session = bluer::Session::new().await?;

    let adapter = session.default_adapter().await?;
    adapter.set_powered(true).await?;
    Ok(adapter)
}

pub async fn bluetooth_adapter() -> Adapter {
    try_bluetooth_adapter
        .retry(
            &ConstantBuilder::default()
                .with_max_times(10)
                .with_delay(Duration::from_secs(1)),
        )
        .await
        .unwrap()
}

pub async fn pretty_device_name(dev: &Device) -> String {
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
