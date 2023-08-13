use bluer::{Adapter, Device};

pub async fn bluetooth_adapter() -> Adapter {
    let session = bluer::Session::new().await.unwrap();
    let adapter = session.default_adapter().await.unwrap();
    adapter.set_powered(true).await.unwrap();
    adapter
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
