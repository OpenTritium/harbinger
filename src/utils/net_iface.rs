use netif::Interface;
use regex::Regex;
use std::sync::OnceLock;
use tokio::sync::{RwLock, RwLockReadGuard};

type LanWan = (Option<Interface>, Option<Interface>);

static INTEREST_IFACE_NAME_PATTERN: OnceLock<regex::Regex> = OnceLock::new();

fn get_perfer_pattern() -> &'static Regex {
    const INTEREST_IFACE_NAME_PATTERN_STR: &str = r"(?xi)
^
(?:
    # WiFi interfaces (traditional + predictable)
    (?:
        w(?:i[-]?fi|lan)\d*            # Traditional: wlan0, wifi0
        |wl[psx]\w+                    # Linux wireless: wlp0s0, wlx001122
        |ww\d+                         # WWAN interfaces
        |(?:iwn|urtw|bwn|ral|run)\d+   # BSD wireless
        |(?:qca|ath|rtl|rtw|mt76|brcm|iw[lmsx])\w+  # Wireless drivers
        |awdl\d+                       # macOS AWDL
    )
  |
    # Ethernet interfaces (traditional + predictable)
    (?:
        eth\d+                         # Traditional Ethernet
        |en(?:                         # Predictable naming:
            [op]\w+                    # eno1 (onboard), enp0s0 (PCI)
            |s\d+                      # ens3 (hotplug slot)
            |x[0-9a-f]{12}             # enx001122334455 (MAC-based)
            |\d+                       # en0 (macOS/fallback)
        )
        |(?:e1000|igb|r816)\w+         # Ethernet drivers
        |(?:em|re|bge|xl|cxgb)\d+      # BSD Ethernet
    )
)
$";
    INTEREST_IFACE_NAME_PATTERN.get_or_init(|| Regex::new(INTEREST_IFACE_NAME_PATTERN_STR).unwrap())
}

static NETWORK_INTERFACE_VIEW: OnceLock<NetworkInterfaceView> = OnceLock::new();
pub struct NetworkInterfaceView {
    cached_interface: RwLock<LanWan>,
}

impl NetworkInterfaceView {
    pub async fn set(&self, lan: Option<Interface>, wan: Option<Interface>) {
        *self.cached_interface.write().await = (lan, wan);
    }

    pub async fn get(&self) -> RwLockReadGuard<LanWan> {
        self.cached_interface.read().await
    }
    /// `true` for global, `false` for link-local
    pub fn get_prefer(global_or_linklocal: bool) -> Option<Interface> {
        let Ok(up_list) = netif::up() else {
            return None;
        };
        up_list.into_iter().find(|iface| {
            get_perfer_pattern().is_match(iface.name())
                && if global_or_linklocal {
                    iface.is_unicast_global()
                } else {
                    iface.is_unicast_link_local()
                }
        })
    }

    pub fn instance() -> &'static NetworkInterfaceView {
        NETWORK_INTERFACE_VIEW.get_or_init(|| NetworkInterfaceView {
            cached_interface: RwLock::new((
                NetworkInterfaceView::get_prefer(false),
                NetworkInterfaceView::get_prefer(true),
            )),
        })
    }
}
