import asyncio
from core.local_ip import LocalIP
from core.multicast import UDPMulticastSocket
from core.controller import Controller
from core.toasts import ToastsFront

if __name__ == '__main__':
    # Ask for default local IP interface.
    interface = LocalIP.get()

    # Start multicast server in thread.
    udpMulticast = UDPMulticastSocket(interface=interface, udp_ip="224.3.29.115", udp_port=45783)
    udpMulticast.start()

    # Start Controller server, providing a way to connect to the service.
    Controller(interface=interface, port=45784).startServer()

    asyncio.run(ToastsFront.showLaunched())

    input()