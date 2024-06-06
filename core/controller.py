from core.toasts import ToastsFront
from aiohttp import web
import threading, socketio
import asyncio
import vgamepad as vg
from asgiref.sync import async_to_sync

class Controller:

    warned_ips = set()
    blacklisted_ips = set()

    interface = "0.0.0.0"
    port = 45784

    gamepads_sid = {}

    loop = asyncio.new_event_loop()

    def __init__(self, interface: str, port: int):
        self.interface = interface
        self.port = port

    def startServer(self):
        sio = socketio.AsyncServer(cors_allowed_origins="*", async_mode='aiohttp', async_handlers=True)
        app = web.Application()
        sio.attach(app)

        @sio.event
        async def connect(sid, environ):
            # Disconnect blacklisted IPs.
            if environ["REMOTE_ADDR"] in self.blacklisted_ips:
                await sio.disconnect(sid)
                return

            # Ask for PIN using toast if no sid is authorized.
            if await ToastsFront.askForPin(title="Controller request",
                                      description="Please enter the PIN code provided on your phone's screen to confirm.",
                                      pin=environ["HTTP_PIN"]):
                if environ["REMOTE_ADDR"] in self.warned_ips:
                    self.warned_ips.remove(environ["REMOTE_ADDR"])
                
                self.gamepads_sid.update({ sid: XboxControllerBridge() })

                @async_to_sync
                async def vib(value):
                    await sio.emit("v", value, to=sid)

                def controller_callback(client, target, large_motor, small_motor, led_number, user_data):
                    if large_motor > 0 or small_motor > 0:
                        vib(True)
                    if large_motor == 0 and small_motor == 0:
                        vib(False)

                self.gamepads_sid[sid].assign_callback(controller_callback)

                asyncio.create_task(ToastsFront.rawToast("Your phone has connected to xPalm", "xPalm is ready to receive inputs from your phone."))
                await sio.emit("authorized", to=sid)

            # Warn if the PIN is wrong.
            elif environ["REMOTE_ADDR"] not in self.warned_ips:
                self.warned_ips.add(environ["REMOTE_ADDR"])
                return await sio.disconnect(sid)

            # If the warned IP requested and the PIN is still wrong, blacklist it.
            else:
                self.warned_ips.remove(environ["REMOTE_ADDR"])
                self.blacklisted_ips.add(environ["REMOTE_ADDR"])
                return await sio.disconnect(sid)
        
        @sio.event
        async def K(sid, isPress, key):
            if isPress:
                self.gamepads_sid[sid].press(key)
            else:
                self.gamepads_sid[sid].release(key)
        
        @sio.event
        async def J(sid, isRight, x, y):
            if isRight:
                self.gamepads_sid[sid].right_joystick(x, y)
            else:
                self.gamepads_sid[sid].left_joystick(x, y)
        
        @sio.event
        async def T(sid, isRight, value):
            if isRight:
                self.gamepads_sid[sid].right_trigger(value)
            else:
                self.gamepads_sid[sid].left_trigger(value)
            
        @sio.event
        async def reset(sid, data):
            self.gamepads_sid[sid].reset()
        
        @sio.event
        async def PI(sid, data):
            await sio.emit("PO", None, to=sid)

        @sio.event
        async def disconnect(sid):
            if sid in self.gamepads_sid:
                self.gamepads_sid[sid].reset()
                self.gamepads_sid[sid].dispose()
                del self.gamepads_sid[sid]
                asyncio.create_task(ToastsFront.rawToast("Your phone has been disconnected from xPalm", "Try reconnecting or double check the internet connection"))
        
        threading.Thread(target=web.run_app, args=(app, ), kwargs={"host": self.interface, "port": self.port}, daemon=True).start()

class XboxControllerBridge:
    gamepad = None

    def __init__(self):
        self.gamepad = vg.VX360Gamepad()

    def assign_callback(self, callback):
        self.gamepad.register_notification(callback_function=callback)

    def press(self, key: vg.XUSB_BUTTON):
        self.gamepad.press_button(key)
        self.gamepad.update()
    
    def release(self, key: vg.XUSB_BUTTON):
        self.gamepad.release_button(key)
        self.gamepad.update()
    
    def left_joystick(self, x: float, y: float):
        self.gamepad.left_joystick_float(x_value_float=x, y_value_float=y)
        self.gamepad.update()
    
    def right_joystick(self, x: float, y: float):
        self.gamepad.right_joystick_float(x_value_float=x, y_value_float=y)
        self.gamepad.update()

    def left_trigger(self, value: float):
        self.gamepad.left_trigger_float(value_float=value)
        self.gamepad.update()
    
    def right_trigger(self, value: float):
        self.gamepad.right_trigger_float(value_float=value)
        self.gamepad.update()
    
    def reset(self):
        self.gamepad.reset()
        self.gamepad.update()
    
    def dispose(self):
        del self.gamepad