use clap::Parser;
use nova_protocol::prelude::*;

#[derive(Parser)]
#[command(name = "nova_protocol")]
#[command(version = "0.1.0")]
#[command(about = "Simple spaceship editor scene where you can build custom ships", long_about = None)]
struct Cli {
    #[cfg(feature = "debug")]
    #[arg(long)]
    debugdump: bool,
    #[cfg(feature = "debug")]
    #[arg(long)]
    norender: bool,
}

fn main() {
    #[allow(unused_variables)]
    let args = Cli::parse();

    let builder = AppBuilder::new();

    #[cfg(feature = "debug")]
    let builder = builder.with_rendering(!args.norender);

    let mut app = builder.build();

    #[cfg(feature = "debug")]
    if args.debugdump {
        debugdump(&mut app);
        std::process::exit(0);
    }

    app.run();
}
