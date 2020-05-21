FROM timescale/timescaledb:latest-pg12
#FROM postgres:alpine
COPY scripts/extensions.sh /docker-entrypoint-initdb.d/