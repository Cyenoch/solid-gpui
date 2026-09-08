use rquickjs::{CatchResultExt, Context, Runtime};

fn main() {
    std::thread::Builder::new().stack_size(8 * 1024 * 1024).spawn(|| {
        let runtime = Runtime::new().unwrap();
        runtime.set_memory_limit(256 * 1024 * 1024);
        runtime.set_max_stack_size(2 * 1024 * 1024);
        let deadline = std::time::Instant::now() + std::time::Duration::from_secs(60);
        runtime.set_interrupt_handler(Some(Box::new(move || std::time::Instant::now() > deadline)));
        let context = Context::full(&runtime).unwrap();
        context.with(|ctx| {
            if let Some(target) = std::env::args().nth(2) {
                ctx.globals().set("probeTarget", target).unwrap();
            }
            let source = std::fs::read(std::env::args().nth(1).expect("bundle path")).unwrap();
            ctx.eval::<(), _>(source).catch(&ctx).unwrap();
            let report: String = ctx.globals().get("probeResult").unwrap();
            println!("{report}");
        });
    }).unwrap().join().unwrap();
}
