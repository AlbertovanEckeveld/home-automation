#!/bin/sh

envsubst < /templates/upsd.users.template > /etc/nut/upsd.users

exec /entrypoint-original.sh