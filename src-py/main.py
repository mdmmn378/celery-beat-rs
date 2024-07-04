from celery import Celery
import time

## poetry add "celery[amqp]"
## docker run -it --rm --name rabbitmq -p 5672:5672 -p 15672:15672 rabbitmq:3.13-management
# app = Celery('tasks', broker='amqp://guest@localhost//', backend='rpc://')

## poetry add "celery[redis]"
## docker run --name redis --rm -p 6379:6379 redis
app = Celery('tasks', broker='redis://localhost:6379/0', backend='redis://localhost:6379/0')

## set json as serializer
app.conf.update(
    task_serializer='json',
    accept_content=['application/json', 'application/x-python-serialize', 'json'],
    result_serializer='json',
)

@app.task
def add(x, y, **kwargs):
    print(f"add({x}, {y}), kwargs: {kwargs}")
    time.sleep(5)
    return x + y


