#!/usr/bin/env bash

# move to temp folder
echo "moving built binaries..."
rm -rf /tmp/dbt-cloud-sync/
mkdir -p /tmp/dbt-cloud-sync/releases
mv target/aarch64-unknown-linux-gnu/release/dbt-cloud-sync /tmp/dbt-cloud-sync/releases/dbt-cloud-sync-aarch64
mv target/x86_64-unknown-linux-gnu/release/dbt-cloud-sync /tmp/dbt-cloud-sync/releases/dbt-cloud-sync-x86_64

# download ghr
echo "cutting github release $CIRCLE_TAG from $CIRCLE_SHA1"
wget https://github.com/tcnksm/ghr/releases/download/v0.16.0/ghr_v0.16.0_linux_amd64.tar.gz -O /tmp/ghr_v0.16.0_linux_amd64.tar.gz
tar -zxvf /tmp/ghr_v0.16.0_linux_amd64.tar.gz -C /tmp/
cp /tmp/ghr_v0.16.0_linux_amd64/ghr /tmp/ghr

# create a new release
/tmp/ghr -u "${CIRCLE_PROJECT_USERNAME}" -r "${CIRCLE_PROJECT_REPONAME}" -c "${CIRCLE_SHA1}" -replace ${CIRCLE_TAG} /tmp/dbt-cloud-sync/releases
