FROM python:3.10-slim as base

WORKDIR /app

RUN apt-get update && apt-get install -y magic libmagic-dev

COPY requirements.txt /app/requirements.txt

RUN pip install --no-cache-dir -r requirements.txt
RUN pip install gunicorn

FROM base as runner

COPY ./app.py /app/app.py

EXPOSE 41890

CMD ["gunicorn", "-b", "0.0.0.0:41890", "app:app"]
