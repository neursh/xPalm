from win11toast import toast_async

class Toasts:
    @staticmethod
    async def showLaunched():
        await toast_async("xPalm is running in the background", "Palm has started a session on this local network and ready to connect. Check out system tray for more options.")

    @staticmethod
    async def askForPin(title: str, description: str, pin: str):
        PIN_REQUEST = await toast_async(title, description, input="PIN", button="Connect")
        if type(PIN_REQUEST) == dict and PIN_REQUEST["user_input"]["PIN"] != "":
            return PIN_REQUEST["user_input"]["PIN"] == pin
        return False
    
    @staticmethod
    async def rawToast(title: str, description: str):
        await toast_async(title, description)