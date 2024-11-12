use std::sync::mpsc::Sender;
use local_ip_address::local_ip;
use std::{ thread, time::Duration };

pub fn fetch(sender: Sender<String>) {
    thread::spawn(move || {
        let mut reported_ip = String::from("");
        loop {
            if let Ok(ip) = local_ip() {
                let ip_str = ip.to_string();
                if ip_str != reported_ip {
                    sender.send(ip_str.clone()).unwrap();
                    reported_ip = ip_str;
                }
            }
            thread::sleep(Duration::from_secs(1));
        }
    });
}
