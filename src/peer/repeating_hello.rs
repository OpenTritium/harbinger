use std::{ops::Deref, thread::sleep, time::Duration};

use crate::{
    addr_v6::{Ipv6Scope, ScopeWithPort},
    msg::Msg,
    utils::{Env, NetworkInterfaceView},
};

pub fn repeating_hello(tx: crate::msg::ParcelSender) {
    tokio::spawn(async move {
        let guard = NetworkInterfaceView::instance().get().await;
        let addr: Ipv6Scope = guard.deref().0.clone().unwrap().try_into().unwrap();
        let Ipv6Scope::Lan { scope_id, .. } = addr else {
            //todo
            return;
        };
        loop {
            tx.send((
                Msg::Hello {
                    host_id: Env::instance().host_id.read().await.clone(),
                    addr,
                }
                .into(),
                Ipv6Scope::Lan {
                    addr: Env::instance().multicast_local.try_into().unwrap(),
                    scope_id,
                },
            ))
            .await
            .unwrap();
            sleep(Duration::from_secs(10));
        }
    });
}
