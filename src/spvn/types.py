from typing import Awaitable, Callable, List, Optional, Tuple, TypedDict


class SendDict(TypedDict):
    """Send is a typed dict representing the fields exposed to clients
    for sending responses back to spvn.

    Args:
        type: str -> "http.response.start"
        status: Optional[int] -> `200`
        message: Optional[str] -> `None` (lifecycle events only)
        headers: Optional[List[Tuple[bytes, bytes]]] -> `[(b'Some-Header', b'value')]`
        body: Optional[bytes] -> `b'Hello World'`
        more_body: Optional[bool] -> `False`
        trailers: Optional[bool] -> `False`
    """

    type: str
    status: Optional[int]
    message: Optional[str]
    headers: Optional[List[Tuple[bytes, bytes]]]
    body: Optional[bytes]
    more_body: Optional[bool]
    trailers: Optional[bool]

Sender = Callable[[SendDict], Awaitable[None]]
