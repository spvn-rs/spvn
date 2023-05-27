async def app(scope, receive=None, send=None):
    print(scope, send)
    awa = send({'key': 'value'})
    print(awa)
    received = receive()
    print(received)
    return 1
print("init")