use redis;
use redis::Commands;

pub fn set_startuptime(time: String) {
//    let client = redis::Client::open("redis://192.168.200.2/").unwrap();
//    let con = client.get_connection().unwrap();
//    let _: () = con.set("startup_time", time);
}

pub fn get_startuptime() -> redis::RedisResult<(String)> {
    let client = redis::Client::open("redis://192.168.200.2/")?;
    let con = client.get_connection()?;

    con.get("startup_time")
}