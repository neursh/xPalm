import socket

class LocalIP:
    @staticmethod
    def get():
        IP = "127.0.0.1"

        tmp = socket.socket(socket.AF_INET, socket.SOCK_DGRAM)
        tmp.settimeout(0)
        
        try:
            tmp.connect(("1.1.1.1", 1))
            IP = tmp.getsockname()[0]
        except:
            pass
        finally:
            tmp.close()

        return IP