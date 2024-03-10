FROM alpine:3.19

WORKDIR /opt/back2back

RUN mkdir -p scripts

RUN apk add --no-cache \
    bash \
    aws-cli \
    postgresql-client

RUN adduser -D -u 1000 -h /opt/back2back -s /bin/bash back2back && \
    chown -R back2back:back2back /opt/back2back

COPY --chown=back2back:back2back scripts/pg-backup-s3.sh /opt/back2back/scripts/backup.sh

# USER mmoreiradj
USER root

# ENTRYPOINT [ "/opt/back2back/scripts/backup.sh" ]
