use netif::Interface;
use regex::Regex;
use std::sync::OnceLock;

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
        vEthernet.+                    # WSL for test
        |eth\d+                        # Traditional Ethernet
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

static NETWORK_INTERFACE_VIEW: OnceLock<NicView> = OnceLock::new();

/// `true` for global, `false` for link-local
pub fn get_prefer(global_or_linklocal: bool) -> Option<Interface> {
    let pattern = Regex::new(INTEREST_IFACE_NAME_PATTERN_STR).unwrap();
    let Ok(up_list) = netif::up() else {
        return None;
    };
    up_list.into_iter().find(|iface| {
        pattern.is_match(iface.name())
            && if global_or_linklocal {
                iface.is_unicast_global()
            } else {
                iface.is_unicast_link_local()
            }
    })
}

type LanWan = [Option<Interface>; 2];

pub struct NicView {
    pub cached_interface: LanWan,
}
// 保证重选网卡后重新绑定socket，通过配置文件读取手动还是自动
//  包装获取所有网卡的方法

impl Default for NicView {
    fn default() -> Self {
        Self {
            cached_interface: [get_prefer(false), get_prefer(true)],
        }
    }
}

pub fn nic_selected() -> &'static LanWan {
    &NETWORK_INTERFACE_VIEW
        .get_or_init(NicView::default)
        .cached_interface
}

