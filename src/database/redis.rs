use redis;
use redis::Commands;

pub fn fetch_an_integer() -> redis::RedisResult<isize> {
    // connect to redis
    let client = try!(redis::Client::open("redis://192.168.200.2/"));
    let con = try!(client.get_connection());
    // throw away the result, just make sure it does not fail
    let _: () = try!(con.set("my_key", 42));
    // read back the key and return it.  Because the return value
    // from the function is a result for integer this will automatically
    // convert into one.
    let count: i32 = try!(con.get("my_key"));
    println!("{}", count);
    con.get("my_key")
}

pub fn set_startuptime(time: String) -> redis::RedisResult<(String)> {
    let client = try!(redis::Client::open("redis://192.168.200.2/"));
    let con = try!(client.get_connection());
    let _: () = try!(con.set("startup_time", time));

//    Ok(())
    con.get("startup_time")
}