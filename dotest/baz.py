import asyncio
from dataclasses import dataclass


@dataclass
class DB:
    cx_id: str


async def dependent_start_task() -> DB:
    return DB(cx_id='123')


async def app(scope=None, receive=None, send=None):
    if scope['type'] == 'lifespan.startup':
        asyncio.run(dependent_start_task())
        return await send(
            {
                'type': 'lifespan.startup.success',
            }
        )
    await receive()
    await send(
        {
            'type': 'http.response.start',
            'status': 200,
            'headers': [
                (b'content-type', b'text/plain'),
            ],
        }
    )
    await send(
        {
            'type': 'http.response.body',
            'body': b'Hello, world!',
        }
    )
