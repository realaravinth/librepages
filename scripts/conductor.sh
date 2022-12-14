#!/bin/bash

readonly NAME=librepages-conductor

docker rm -f $NAME

docker create --name $NAME -p 5000:5000 \
	-e LPCONDUCTOR__SOURCE_CODE="https://git.batsense.net/LibrePages/conductor" \
	-e LPCONDUCTOR_SERVER__PROXY_HAS_TLS=false \
	-e LPCONDUCTOR_DEBUG="false" \
	-e LPCONDUCTOR_CONDUCTOR="dummy" \
	-e LPCONDUCTOR_SERVER_URL_PREFIX="" \
	-e LPCONDUCTOR_SERVER_DOMAIN="librepages.test" \
	-e LPCONDUCTOR_SERVER_IP="0.0.0.0" \
	-e LPCONDUCTOR_SERVER_PROXY_HAS_TLS="false" \
	-e LPCONDUCTOR_SERVER_PORT=7000 \
	-e LPCONDUCTOR_SOURCE_CODE="https://example.org" \
    -e LPCONDUCTOR_CREDS_USERNAME=$LPCONDUCTOR_CREDS_USERNAME \
	-e LPCONDUCTOR_CREDS_PASSWORD=$LPCONDUCTOR_CREDS_PASSWORD \
    -e PORT="5000"\
	realaravinth/librepages-conductor conductor serve

docker start $NAME
