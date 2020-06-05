FROM postgres:latest
COPY scripts/extensions.sql /docker-entrypoint-initdb.d/
RUN chmod 644 /docker-entrypoint-initdb.d/extensions.sql