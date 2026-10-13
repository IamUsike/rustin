use mini_redis::{Result, client};

#[tokio::main]
async fn main() -> Result<()> {
    //opem a connection to mini redis
    let mut client = client::connect("127.0.0.1:6379").await?;

    //set kvp
    client.set("hello", "world".into()).await?;

    //get the key
    let res = client.get("hello").await?;

    println!("got value: res = {:?}", res);

    Ok(())
}
