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

fn fib(n: u64) -> u64 {
    if n == 1 {
        return 1;
    } else if n == 0 {
        return 0;
    } else {
        fib(n - 1) + fib(n - 2)
    }
}
