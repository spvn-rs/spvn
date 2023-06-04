import asyncio
from fastapi import FastAPI

app = FastAPI()


@app.get("/")
async def run_something():
    return "ok"


async def send(a):
    print('send', a)


async def receive(**a):
    print('recv', a)
    pass


if __name__ == "__main__":
    asyncio.run(app({"type": "lifespan"}, receive, send))
