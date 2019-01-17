use eval::{eval, to_value};

command!(multiply(_ctx, msg, args) {
    let one = args.single::<f64>().unwrap();
    let two = args.single::<f64>().unwrap();

    let product = one * two;

    let _ = msg.channel_id.say(product);
});

command!(fibonacci(_ctx, msg, args) {
    let n = args.single::<u128>().unwrap();
    let _ = msg.channel_id.say(fib(n));
});

command!(calc(_ctx, msg, args) {
    if let Ok(value) = eval(&args.full()) {
        info!("{}", value);
        if value.is_null() {
            let _ = msg.channel_id.say("Could not parse parameters");
        } else {
            let _ = msg.channel_id.say(value);
        }
    } else {
        let _ = msg.channel_id.say("Could not parse parameters");
    }
});

fn fib(n: u128) -> u128 {
    let mut a = 0u128;
    let mut b = 1u128;
    let mut c = 0u128;

    if n == 0 {
        return 0;
    }

    for _ in 0..(n - 1) {
        c = a + b;
        a = b;
        b = c;
    }
    return b;
}
