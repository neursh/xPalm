mod services;
mod utils;

use std::{
    collections::HashMap,
    net::{ IpAddr, Ipv4Addr, SocketAddr },
    str::FromStr,
    sync::{ mpsc, Arc },
};
use tokio::sync::Mutex;
use colored::Colorize;
use services::{ announcer, instance, local_ip };
use utils::clear;
use vigem_client::{ Client, XGamepad, Xbox360Wired };
use whoami::fallible;

#[tokio::main]
async fn main() {
    let hostname = fallible::hostname().unwrap();

    let (ip_sender, ip_receiver) = mpsc::channel::<String>();
    local_ip::fetch(ip_sender);

    let mut announcer_task: Option<
        tokio::task::JoinHandle<Result<(), std::io::Error>>
    > = None;
    let mut joystick_task: Option<
        tokio::task::JoinHandle<Result<(), std::io::Error>>
    > = None;
    let mut manager_task: Option<
        tokio::task::JoinHandle<Result<(), std::io::Error>>
    > = None;

    let mut current_ip = ip_receiver.recv().unwrap();

    loop {
        clear::invoke();
        println!(
            "{} Binding instance on IP Address: {}",
            ">".green(),
            current_ip.bright_cyan()
        );
        println!(
            "{} Manual connect information | IP: {} | Port: {}",
            ">".green(),
            current_ip.bright_cyan(),
            "45784".bright_cyan()
        );

        if let Some(task) = announcer_task {
            task.abort();
        }
        if let Some(task) = joystick_task {
            task.abort();
        }
        if let Some(task) = manager_task {
            task.abort();
        }

        let controller_list: Arc<
            Mutex<HashMap<IpAddr, (Xbox360Wired<Arc<Client>>, XGamepad)>>
        > = Arc::new(Mutex::new(HashMap::new()));
        let blocked: Arc<Mutex<Vec<IpAddr>>> = Arc::new(Mutex::new(Vec::new()));

        let host_v4 = Ipv4Addr::from_str(&current_ip).unwrap();
        let host_addr = IpAddr::V4(host_v4);

        let vigem = Arc::new(Client::connect().unwrap());

        let announcer_target = SocketAddr::new(host_addr, 45783);
        announcer_task = Some(
            tokio::spawn(
                announcer::start(
                    announcer_target.clone(),
                    host_addr.clone(),
                    host_v4.clone(),
                    hostname.clone()
                )
            )
        );

        let manager_target = SocketAddr::new(host_addr, 45784);
        let mut manager_controller_list = controller_list.clone();
        let mut manager_blocked = blocked.clone();
        manager_task = Some(
            tokio::spawn(async move {
                instance::launch_main(
                    manager_target,
                    &mut manager_controller_list,
                    &mut manager_blocked,
                    vigem.clone()
                ).await
            })
        );

        let joystick_target = SocketAddr::new(host_addr, 45784);
        let mut joystick_controller_list = controller_list.clone();
        joystick_task = Some(
            tokio::spawn(async move {
                instance::launch_joystick(
                    joystick_target,
                    &mut joystick_controller_list
                ).await
            })
        );

        current_ip = ip_receiver.recv().unwrap();
    }
}
