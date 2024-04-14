#!/bin/bash

cargo build --release --target wasm32-unknown-unknown
wasm-bindgen --no-typescript --target web \
    --out-dir ./out/ \
    --out-name "ld55" \
    ./target/wasm32-unknown-unknown/release/ld55.wasm

cp -r assets out/

# remove things that are not needed
rm out/*aseprite

OUTFILE=out/ld55_bg.wasm
# echo "optimizing wasm..."
# wasm-opt -Os $OUTFILE -o test.wasm
# mv test.wasm $OUTFILE

cat << EOF > out/index.html
<!doctype html>
<html lang="en">

<body style="margin: 0px;">
  <script type="module">
    import init from './ld55.js'

    init().catch((error) => {
      if (!error.message.startsWith("Using exceptions for control flow, don't mind me. This isn't actually an error!")) {
        throw error;
      }
    });
  </script>
</body>
<h1>Raccoon Tycoon</h1>
This game may take a minute to load on a fast connection. Your keyboard input won't be captured until after your first click.
</html>
EOF

aws s3 sync out/ s3://www.50ply.com/ld55/preview/ --profile 50ply --acl public-read