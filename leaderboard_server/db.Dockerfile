FROM timescale/timescaledb:latest-pg12
COPY scripts/extensions.sql /docker-entrypoint-initdb.d/
RUN chmod 644 /docker-entrypoint-initdb.d/extensions.sql