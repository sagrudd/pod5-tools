FROM python:3.12-slim

ARG POD5_PACKAGE=pod5

RUN python -m pip install --no-cache-dir --upgrade pip \
    && python -m pip install --no-cache-dir "${POD5_PACKAGE}"

WORKDIR /pod5-tools
