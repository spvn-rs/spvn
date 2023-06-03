async def app(scope=None, receive=None, send=None, rcv=None):
    received = await receive()
    awa2 = await send(
        {"type": "http.response.body", "headers": [(b"content-length", b"8")], "body": b"okok", 'more_body': True}
    )
    awa2 = await send({"type": "http.response.body", "body": b"okok", 'more_body': False})
    return 1
