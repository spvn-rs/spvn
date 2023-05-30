from typing import TypedDict

__err__ = "should not be called directly, instead you may use it to provide type annotations to your project"


class Send(TypedDict):
    """Send is a typed dict representing the fields exposed to clients
    for sending responses back to spvn. All fields MUST be included, specifying
    None value when applicable.

    Args:
        type: str -> "http.response.start"
        headers: list[tuple[str, bytes]] -> [("x-content", b'abc-value')]
        body: bytes | None -> b"my response"
    """

    type: str
    headers: list[tuple[str, bytes]]
    body: bytes | None


class Sender:
    def __call__(self, obj: Send) -> None:
        raise NotImplementedError(__err__)
