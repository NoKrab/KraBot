use eval::{eval, to_value};

command!(multiply(_ctx, msg, args) {
    let one = args.single::<f64>().unwrap();
    let two = args.single::<f64>().unwrap();

    let product = one * two;

    let _ = msg.channel_id.say(product);
});

command!(fibonacci(_ctx, msg, args) {
    let n = args.single::<u64>().unwrap();
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

fn fib(n: u64) -> u64 {
    match n {
        0 => 1,
        1 => 1,
        n => fib(n-1) + fib(n-2),
    }
}
