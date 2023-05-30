async def app(scope=None, receive=None, send=None):
    print("from python",scope, send, receive)
    print("init")

    # print(awa)
    received = receive()
    print("request", received)
    # awa = send({'type': 'http.response.start', 'headers': [], 'body': None})

    awa = send({'type': 'http.response.body', 'headers': (), 'body': b'okok'})
    return 1
# print("init")