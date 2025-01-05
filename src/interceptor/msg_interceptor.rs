use crate::msg::msg_socket::msg_socket;
use crate::solution::peer_event_executor::peer_event_executor;
use std::sync::OnceLock;
use tokio_stream::StreamExt;

static MSG_INTERCEPTOR_INSTANCE:OnceLock<MsgInterceptor> = OnceLock::new();
pub fn msg_interceptor() -> &'static MsgInterceptor {
    MSG_INTERCEPTOR_INSTANCE.get_or_init(|| MsgInterceptor::new())
}
pub struct MsgInterceptor {
}
// 过滤已连接用户的请求连接组播hello报文
impl MsgInterceptor {
    
    pub fn bridge_and_filtering(&self){
        // 过滤 msg
        // 将 msg 映射到 state

        println!("获取到邻居事件执行器的发送器");
        tokio::spawn(async move {
            let txc = peer_event_executor().listening();
            while let Some(msg) = msg_socket().msg_streaming().next().await {
                txc.send(msg.into()).await.unwrap();
                println!("将消息流接收到的消息发送到邻居事件执行器");
            }
            panic!("小溪流解析失败");
        });
    }
    pub fn new()->Self{
        println!("{}","消息拦截器初始化");
        Self{
        }
    }
}