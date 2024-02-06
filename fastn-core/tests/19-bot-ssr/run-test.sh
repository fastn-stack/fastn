#!/bin/bash

FASTN="$FBT_CWD/../target/release/fastn"
SITE_URL="http://127.0.0.1:8000"
BOT_USER_AGENT="Mozilla/5.0 (compatible; Googlebot/2.1; +http://www.google.com/bot.html)"
NORMAL_USER_AGENT="Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/123 Safari/537.36"

"$FASTN" --test update

echo "Starting fastn server ..."
"$FASTN" --test serve --offline &
FASTN_SERVE_PID=$!

sleep 5

build_dir=".build"
if [ ! -d "$build_dir" ]; then
    mkdir "$build_dir"
fi

echo "Curling $SITE_URL with User Agent $BOT_USER_AGENT ..."
curl -A "$BOT_USER_AGENT" -s "$SITE_URL" -o "$build_dir/index-with-ssr.html"

sleep 5

echo "Curling $SITE_URL with User Agent $NORMAL_USER_AGENT ..."
curl -A "$NORMAL_USER_AGENT" -s "$SITE_URL" -o "$build_dir/index-without-ssr.html"

sleep 5

kill -9 "$FASTN_SERVE_PID"
