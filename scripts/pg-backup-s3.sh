#!/bin/bash

set -e

pg_dump_name="pg_dumpall-$(date '+%Y-%m-%d-%H-%M')"

echo "Dumping all databases to $pg_dump_name"

pg_dumpall --clean --if-exists --load-via-partition-root --quote-all-identifiers --no-password --file="$PGDUMP_DIR/$pg_dump_name"

# TODO: encrypt the dump

echo "Uploading to S3"

aws configure set aws_access_key_id "$AWS_ACCESS_KEY_ID"
aws configure set aws_secret_access_key "$AWS_SECRET_ACCESS_KEY"
aws configure set default.region "$AWS_DEFAULT_REGION"
aws configure set default.s3.signature_version "s3v4"

if [ -z "$S3_ENDPOINT" ]; then
  aws s3 cp "$PGDUMP_DIR/$pg_dump_name" "s3://$S3_BUCKET/$pg_dump_name"
else
  aws s3 cp "$PGDUMP_DIR/$pg_dump_name" "s3://$S3_BUCKET/$pg_dump_name" --endpoint-url "$S3_ENDPOINT"
fi
