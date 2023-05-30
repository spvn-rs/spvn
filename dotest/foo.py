async def app(scope=None, receive=None, send=None):
    print("from python", scope, send, receive)
    received = receive()
    print("request", received)
    awa1 = send({'type': 'http.response.start', 'headers': [], 'body': None})
    print("awaitable-1", awa1)
    awa2 = send({"type": "http.response.body", "headers": [("a", b"b")], "body": b"okok"})
    print("awaitable-2", awa2)
    return 1
