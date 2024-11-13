use std::sync::Arc;
use colored::Colorize;
use keyboard_listener_windows::Event;
use vigem_client::{ Client, TargetId, Xbox360Wired };

pub fn listen(vigem: Arc<Client>) {
    let mut ghost_controllers: Vec<Xbox360Wired<Arc<Client>>> = vec![];

    keyboard_listener_windows::start_listen(move |event| {
        callback(event, vigem.clone(), &mut ghost_controllers);
    });
}

fn callback(
    event: Event,
    vigem: Arc<Client>,
    ghost_controllers: &mut Vec<Xbox360Wired<Arc<Client>>>
) {
    if !event.is_key_down {
        if event.key == "F7" {
            let mut controller = Xbox360Wired::new(
                vigem.clone(),
                TargetId::XBOX360_WIRED
            );
            controller.plugin().unwrap();
            controller.wait_ready().unwrap();

            ghost_controllers.push(controller);

            println!(
                "{} Added a ghost controller. ({})",
                ">".green(),
                ghost_controllers.len().to_string().bright_cyan()
            );
        }
        if event.key == "F8" {
            if !ghost_controllers.pop().is_none() {
                println!(
                    "{} Removed a ghost controller. ({})",
                    ">".yellow(),
                    ghost_controllers.len().to_string().bright_cyan()
                );
            }
        }
    }
}
