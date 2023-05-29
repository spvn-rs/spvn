async def app(scope=None, receive=None, send=None):
    print(scope, send, receive)
    awa = send({'key': 'value'})
    print(awa)
    received = receive()
    print(received)
    return 1
print("init")