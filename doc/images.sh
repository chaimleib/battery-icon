#!/bin/bash
grep '^battery-icon --level' README.md |
  PATH=target/debug /bin/bash

# Convert to PNG, since the SVG with FontAwesome won't render in markdown.
for f in doc/*.svg; do
  magick -background none "$f" "${f%.svg}.png"
  rm "$f"
done
