use clap::Parser;

use bitwarden_pin::{
    brute_force_pin,
    cli::Args,
    log::{error, info, success},
};

fn main() {
    let args = Args::parse();
    let kdf_config = args.kdf_config.unwrap_or_default();

    info(&format!("KDF Configuration: {kdf_config:#?}"));

    let max = 10usize.pow(args.pin_length as u32);
    // zero-padded pin strings
    let pins = (0..max).map(|pin| format!("{pin:0length$}", length = args.pin_length));

    info(&format!(
        "Brute forcing PIN from '{:0length$}' to '{:0length$}'...",
        0,
        max - 1,
        length = args.pin_length
    ));
    if let Some(pin) = brute_force_pin(&args.encrypted, &args.email, kdf_config, pins, Some(max)) {
        success(&format!("Pin found: {pin}"));
    } else {
        error("Pin not found");
    }
}
