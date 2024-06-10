from win11toast import toast_async, notify

def showLaunched():
    notify("xPalm is running", "xPalm has started a session on this local network and ready to connect.")

async def askForPin(pin: str):
    global result
    result = None

    buttons = [
        {"activationType": "protocol", "arguments": "http:accept", "content": "Accept"},
        {"activationType": "protocol", "arguments": "http:decline", "content": "Decline"}
    ]

    def receive(args):
        global result
        result = args["arguments"].replace("http:", "")

    await toast_async("Incoming request", f"A device requested a connection with PIN {pin}", buttons=buttons, on_click=receive)

    return result == "accept"

def rawToast(title: str, description: str):
    notify(title, description)
