use std::time::SystemTime;

command!(multiply(_ctx, msg, args) {
    let one = args.single::<f64>().unwrap();
    let two = args.single::<f64>().unwrap();

    let product = one * two;

    let _ = msg.channel_id.say(product);
});

command!(fibonacci(_ctx, msg, args) {
    let n = args.single::<u32>().unwrap();
    let now = SystemTime::now();
    let fib = fib(n);
    let after = SystemTime::now();
    let difference = after.duration_since(now)
                            .expect("SystemTime::duration_since failed");
    let msg_content = format!("F({}) = {}; {:?}", n, fib, difference);
    let _ = msg.channel_id.say(msg_content);
});

fn fib(n: u64) -> u64 {
    if n == 1 {
        return 1;
    } else if n == 0 {
        return 0;
    } else {
        fib(n - 1) + fib(n - 2)
    }
}
