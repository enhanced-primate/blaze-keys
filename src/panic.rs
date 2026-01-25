use std::fs::File;
use std::io::Write;

pub fn register_hook() {
    let hook = std::panic::take_hook();

    // If we panic during the TUI render, the message may be invisible to the user.
    // We write the error to a file instead.
    std::panic::set_hook(Box::new(move |info| {
        let location = info.location().unwrap();
        let message = info.payload().downcast_ref::<&str>();

        let out = if let Some(message) = message {
            &format!("Message: {}", message)
        } else {
            "Panic occurred without a message."
        };

        let mut file = File::create(".panic.blz").unwrap();
        write!(
            file,
            "A panic occurred in blz: \n{out:?}\nlocation: {location:?}"
        )
        .unwrap();

        eprintln!("Panicked! (location={location}) \nmessage={message:?}");
        hook(info);
    }));
}
