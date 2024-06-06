# import vgamepad as vg
# import time

# gamepad = vg.VX360Gamepad()

# time.sleep(1)

# gamepad.press_button(vg.XUSB_BUTTON.XUSB_GAMEPAD_DPAD_UP)
# gamepad.press_button(vg.XUSB_BUTTON.XUSB_GAMEPAD_DPAD_DOWN)
# gamepad.press_button(vg.XUSB_BUTTON.XUSB_GAMEPAD_DPAD_LEFT)
# gamepad.press_button(vg.XUSB_BUTTON.XUSB_GAMEPAD_DPAD_RIGHT)
# gamepad.press_button(vg.XUSB_BUTTON.XUSB_GAMEPAD_START)
# gamepad.press_button(vg.XUSB_BUTTON.XUSB_GAMEPAD_BACK)
# gamepad.press_button(vg.XUSB_BUTTON.XUSB_GAMEPAD_LEFT_THUMB)
# gamepad.press_button(vg.XUSB_BUTTON.XUSB_GAMEPAD_RIGHT_THUMB)
# gamepad.press_button(vg.XUSB_BUTTON.XUSB_GAMEPAD_LEFT_SHOULDER)
# gamepad.press_button(vg.XUSB_BUTTON.XUSB_GAMEPAD_RIGHT_SHOULDER)
# gamepad.press_button(vg.XUSB_BUTTON.XUSB_GAMEPAD_GUIDE)
# gamepad.press_button(vg.XUSB_BUTTON.XUSB_GAMEPAD_A)
# gamepad.press_button(vg.XUSB_BUTTON.XUSB_GAMEPAD_B)
# gamepad.press_button(vg.XUSB_BUTTON.XUSB_GAMEPAD_X)
# gamepad.press_button(vg.XUSB_BUTTON.XUSB_GAMEPAD_Y)

# gamepad.left_trigger_float(value_float=1.0)
# gamepad.right_trigger_float(value_float=1.0)

# gamepad.left_joystick_float(x_value_float=-0.5, y_value_float=0.0)
# gamepad.right_joystick_float(x_value_float=-1.0, y_value_float=0.8)

# gamepad.update()

# input()

import asyncio
from core.local_ip import LocalIP
from core.multicast import UDPMulticastSocket
from core.controller import Controller
from core.toasts import Toasts

if __name__ == '__main__':
    # Ask for default local IP interface.
    interface = LocalIP.get()

    # Start multicast server in thread.
    udpMulticast = UDPMulticastSocket(interface=interface, udp_ip="224.3.29.115", udp_port=45783)
    udpMulticast.start()

    # Start PhoneController server, providing a way to connect to the service.
    Controller(interface=interface, port=45784).startServer()

    asyncio.run(Toasts.showLaunched())

    input()