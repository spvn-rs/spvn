import asyncio
from fastapi import FastAPI
from contextlib import asynccontextmanager
ml_models = {}


@asynccontextmanager
async def lifespan(app: FastAPI):
    ml_models["answer_to_everything"] = 42
    print("set", app)

    yield
    print("model clearing")
    ml_models.clear()


app = FastAPI(lifespan=lifespan)


@app.get("/")
async def run_something():
    return {"resp": "ok"}


def fake_answer_to_everything_ml_model(x: float):
    return x * 42

@app.get("/download")
async def run_something():
    return open("./dotest/canada.json", 'rb').read()

@app.get("/state")
async def run_something():
    return {"resp": ml_models.get("answer_to_everything", "actual nothing")}



async def send(a):
    pass
async def receive(**a):
    pass

if __name__ == "__main__":
    asyncio.run(app({"type": "lifespan"}, receive, send))
