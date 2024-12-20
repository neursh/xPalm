use colored::Colorize;
use tokio::{
    io::{ AsyncReadExt, AsyncWriteExt },
    net::{ TcpListener, UdpSocket },
    sync::{ Mutex, RwLock },
};
use vigem_client::{ Client, TargetId, XGamepad, Xbox360Wired };
use std::{ collections::HashMap, io, net::{ IpAddr, SocketAddr }, sync::Arc };

pub async fn launch_main(
    host_target: SocketAddr,
    controller_list: &mut Arc<
        RwLock<HashMap<IpAddr, Mutex<(Xbox360Wired<Arc<Client>>, XGamepad)>>>
    >,
    blocked: &mut Arc<Mutex<Vec<IpAddr>>>,
    vigem: Arc<Client>
) -> io::Result<()> {
    let sock = TcpListener::bind(host_target).await.unwrap();

    loop {
        let mut client = sock.accept().await.unwrap();

        {
            let lock_blocked = blocked.lock().await;

            if lock_blocked.contains(&client.1.ip()) {
                client.0.shutdown().await.unwrap();
            }
        }

        let arc_controller_list = controller_list.clone();
        let arc_client = Arc::new(Mutex::new(client));
        tokio::spawn(
            client_manager(
                vigem.clone(),
                arc_client.clone(),
                arc_controller_list
            )
        );
    }
}

pub async fn client_manager(
    vigem: Arc<Client>,
    client: Arc<Mutex<(tokio::net::TcpStream, SocketAddr)>>,
    controller_list: Arc<
        RwLock<HashMap<IpAddr, Mutex<(Xbox360Wired<Arc<Client>>, XGamepad)>>>
    >
) -> io::Result<()> {
    let mut lock_client = client.lock().await;

    loop {
        let mut buf = [0; 4];

        if let Ok(size) = lock_client.0.read(&mut buf).await {
            if size == 0 {
                {
                    println!(
                        "{} {} disconnected.",
                        ">".yellow(),
                        lock_client.1.ip().to_string().bright_cyan()
                    );
                    {
                        let mut write_controller_list =
                            controller_list.write().await;
                        write_controller_list.remove(&lock_client.1.ip());
                    }
                }
                lock_client.0.shutdown().await.unwrap();
                break Ok(());
            }
        } else {
            break Ok(());
        }

        if buf[0] == 0 {
            if
                let Some(result) = request_prompt(
                    lock_client.1,
                    vigem.clone()
                ).await
            {
                {
                    let mut write_controller_list =
                        controller_list.write().await;
                    write_controller_list.insert(lock_client.1.ip(), result);
                }

                lock_client.0.write(&[1]).await.unwrap();
            } else {
                lock_client.0.shutdown().await.unwrap();
                break Ok(());
            }
            continue;
        }

        {
            let read_controller_list = controller_list.read().await;
            if read_controller_list.contains_key(&lock_client.1.ip()) {
                let mut control = read_controller_list
                    .get(&lock_client.1.ip())
                    .unwrap()
                    .lock().await;
                if buf[0] == 2 {
                    if buf[1] == 1 {
                        control.1.buttons.raw |= u16::from_le_bytes(
                            buf[2..4].try_into().unwrap()
                        );
                    }
                    if buf[1] == 0 {
                        control.1.buttons.raw &= !u16::from_le_bytes(
                            buf[2..4].try_into().unwrap()
                        );
                    }

                    let pads = control.1;
                    let _ = control.0.update(&pads);
                    continue;
                }

                if buf[0] == 4 {
                    if buf[1] == 0 {
                        control.1.left_trigger = buf[3] * 255;
                    }
                    if buf[1] == 1 {
                        control.1.right_trigger = buf[3] * 255;
                    }

                    let pads = control.1;
                    let _ = control.0.update(&pads);
                    continue;
                }

                if buf[0] == 6 {
                    lock_client.0.write(&[6, 0, 0, 0]).await?;
                }
            } else {
                break Ok(());
            }
        }
    }
}

pub async fn launch_joystick(
    host_target: SocketAddr,
    controller_list: &mut Arc<
        RwLock<HashMap<IpAddr, Mutex<(Xbox360Wired<Arc<Client>>, XGamepad)>>>
    >
) -> io::Result<()> {
    let sock = UdpSocket::bind(host_target).await.unwrap();

    let mut buf = [0; 6];
    loop {
        let receiver = if let Ok(result) = sock.recv_from(&mut buf).await {
            result
        } else {
            continue;
        };

        {
            let read_controller_list = controller_list.read().await;
            if read_controller_list.contains_key(&receiver.1.ip()) {
                let mut control = read_controller_list
                    .get(&receiver.1.ip())
                    .unwrap()
                    .lock().await;
                if buf[0] == 3 {
                    if buf[1] == 0 {
                        control.1.thumb_lx = i16::from_le_bytes(
                            buf[2..4].try_into().unwrap()
                        );
                        control.1.thumb_ly = i16::from_le_bytes(
                            buf[4..6].try_into().unwrap()
                        );
                    }
                    if buf[1] == 1 {
                        control.1.thumb_rx = i16::from_le_bytes(
                            buf[2..4].try_into().unwrap()
                        );
                        control.1.thumb_ry = i16::from_le_bytes(
                            buf[4..6].try_into().unwrap()
                        );
                    }

                    let joysticks = control.1;
                    let _ = control.0.update(&joysticks);
                    continue;
                }
            }
        }
    }
}

async fn request_prompt<'a>(
    client_ip: SocketAddr,
    vigem: Arc<Client>
) -> Option<Mutex<(Xbox360Wired<Arc<Client>>, XGamepad)>> {
    let response = inquire::Select
        ::new(
            &format!(
                "Incoming request from {}:",
                client_ip.ip().to_string().bright_cyan()
            ),
            ["Accept", "Reject", "Block (For the whole instance)"].to_vec()
        )
        .prompt()
        .unwrap();

    if response == "Accept" {
        let mut controller = Xbox360Wired::new(
            vigem.clone(),
            TargetId::XBOX360_WIRED
        );
        controller.plugin().unwrap();
        controller.wait_ready().unwrap();

        let gamepad = vigem_client::XGamepad::default();

        return Some(Mutex::new((controller, gamepad)));
    }

    None
}
