from toasted import Toast, Text, Button
from toasted.enums import ToastButtonStyle, ToastTextStyle, ToastTextAlign

APP_ID = Toast.register_app_id("Neurs.Dev.xPalm", "xPalm")

class ToastsFront:
    @staticmethod
    async def showLaunched():
        toast = Toast(app_id=APP_ID)
        toast.elements = [
            Text("xPalm is running in the background"),
            Text("Palm has started a session on this local network and ready to connect.")
        ]
        
        await toast.show()

    @staticmethod
    async def askForPin(title: str, description: str, pin: str):
        toast = Toast(app_id=APP_ID)
        toast.elements = [
            Text(title),
            Text(description),
            [
                [
                    Text(pin, style=ToastTextStyle.HEADER, align=ToastTextAlign.CENTER),
                ]
            ],
            Button("Decline", arguments="decline", style=ToastButtonStyle.CRITICAL),
            Button("Accept", arguments="accept", style=ToastButtonStyle.SUCCESS),
        ]

        result = await toast.show()

        return result.arguments == "accept"
    
    @staticmethod
    async def rawToast(title: str, description: str):
        toast = Toast(app_id=APP_ID)
        toast.elements = [
            Text(title),
            Text(description),
        ]

        await toast.show()