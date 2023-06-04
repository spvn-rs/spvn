from fastapi import FastAPI

import asyncio
app = FastAPI()


@app.get("/")
async def run_something():
    return "ok"


async def send(a):
    # raise ValueError(a)
    print('send', a)

async def receive(**a):
    print('recv', a)

    pass


if __name__ == "__main__":
    asyncio.run(
    app({
        "type":"lifespan"
    }, receive, send)
    )
    