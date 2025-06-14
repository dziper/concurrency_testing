use crate::funcs::my_async_func;

pub async fn thingy() {
    my_async_func().await;
}