FROM cgr.dev/chainguard/static

WORKDIR /app

COPY --chown=nonroot:nonroot controller /app/controller

EXPOSE 8080

ENTRYPOINT [ "/app/controller" ]
