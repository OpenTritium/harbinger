use std::sync::Arc;

use dashmap::{mapref::one::RefMut, DashMap};

use crate::uid::Uid;

use super::future_state::PeerFutureState;

pub struct Cursor {
    peers: Arc<DashMap<Uid, PeerFutureState>>,
    keys: Vec<Uid>,
    current_index: usize,
}

impl Cursor {
    pub fn new(peers: Arc<DashMap<Uid, PeerFutureState>>) -> Self {
        Cursor {
            keys: peers
                .iter()
                .map(|record| record.key().clone())
                .collect::<Vec<_>>(),
            current_index: 0,
            peers,
        }
    }
    pub fn get_next_record(&mut self) -> RefMut<'_, Uid, PeerFutureState> {
        // 索引超出就重新生成索引
        if let Some(uid) = self.keys.get(self.current_index) {
            // 记得处理索引与表数据不同步导致的无法查找错误
            self.peers.get_mut(uid).unwrap()
        } else {
            self.keys = self
                .peers
                .iter()
                .map(|record| record.key().clone())
                .collect::<Vec<_>>();
            self.current_index = 0;
            self.get_next_record()
        }
    }
}
