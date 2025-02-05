use crate::utils::env;
use crate::{addr_v6::Ipv6Scope, msg::Msg, utils::nic_selected};
use anyhow::{Result, anyhow};
use tracing::info;
use std::time::Duration;

pub async fn repeating_hello(tx: crate::msg::ParcelSender) -> Result<()> {
    nic_selected()[0]
        .as_ref()
        .map(|iface| {
            info!("hello 中");
            tokio::spawn(async move {
                let src: Ipv6Scope = iface.try_into().unwrap();
                let dest = Ipv6Scope::Lan {
                    addr: env().multicast_lan.try_into().unwrap(),
                    scope_id: src.scope_id().unwrap(),
                };
                loop {
                    tx.clone()
                        .send((
                            Msg::Hello {
                                host_id: env().host_id.read().await.clone(),
                                addr: src,
                            },
                            dest,
                        ))
                        .await
                        .unwrap();
                    tokio::time::sleep(Duration::from_secs(5)).await;
                }
            });
        })
        .ok_or(anyhow!(""))
}
