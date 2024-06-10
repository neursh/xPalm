from sys import exit
from core.local_ip import LocalIP
from core.multicast import UDPMulticastSocket
from core.controller import Controller
from core.toasts import showLaunched

if __name__ == '__main__':
    # Ask for default local IP interface.
    interface = LocalIP.get()

    # Start multicast server in thread.
    UDPMulticastSocket(interface=interface, udp_ip="224.3.29.115", udp_port=45783).start()

    # Start Controller server, providing a way to connect to the service.
    Controller(interface=interface, port=45784).start()

    showLaunched()

    try:
        input()
    except KeyboardInterrupt:
        exit(0)