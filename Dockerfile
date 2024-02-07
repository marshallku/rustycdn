FROM python:3.10-slim

WORKDIR /app
COPY . /app

RUN apt-get update && apt-get install -y magic libmagic-dev
RUN pip install --no-cache-dir -r requirements.txt
RUN pip install gunicorn

EXPOSE 41890

CMD ["gunicorn", "-b", "0.0.0.0:41890", "app:app"]
