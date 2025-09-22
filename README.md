# battery-icon

Customize a battery icon SVG with a charge bar and charging icon.

```bash
battery-icon --level 1 --charging base-src.svg doc/level100-charging.svg
```
![fully-charged, plugged in](./doc/level100-charging.png)

```bash
battery-icon --level 1 base-src.svg doc/level100-notcharging.svg
```
![fully-charged, not plugged in](./doc/level100-notcharging.png)

```bash
battery-icon --level 0.5 base-src.svg doc/level50-notcharging.svg
```
![50% charged, not plugged in](./doc/level50-notcharging.png)

```bash
battery-icon --level 0.2 base-src.svg doc/level20-notcharging.svg
```
![20% charged, not plugged in](./doc/level20-notcharging.png)

```bash
battery-icon --level 0.10 base-src.svg doc/level10-notcharging.svg
```
![10% charged, not plugged in](./doc/level10-notcharging.png)

```bash
battery-icon --level 1 --charging --foreground aa0000 base-src.svg doc/level100-charging-red.svg
```
![fully-charged, plugged in, red text](./doc/level100-charging-red.png)

## Install

```bash
cargo build --release
sudo cp -v target/release/battery-icon /usr/local/bin
sudo mkdir /usr/local/share/battery-icon
sudo cp -v base-src.svg /usr/local/share/battery-icon/base-src.svg
```

## Usage with hyprlock

This program was created for use with
[hyprlock](https://wiki.hypr.land/Hypr-Ecosystem/hyprlock/).

```bash
#!/bin/bash
# ~/.config/hypr/battery-icon.sh

# Gather battery status vars.
battery_data=$(upower -i "$(upower -e | grep bat)")
percent=$(
  echo "$battery_data" |
    sed -nE 's/^\s+percentage:\s+([[0-9.]+)%/\1/p'
)
level=$(printf 'scale = 3; %d/100\n' "$percent" | bc)
discharging=$(echo "$battery_data" | grep -E '^\s+state:\s+discharging')
if [ -z "$discharging" ]; then
  charging=--charging
else
  unset charging
fi

# Generate the SVG.
battery-icon \
  --foreground "ffffff" \
  --level "$level" \
  $charging \
  /usr/local/share/battery-icon/base-src.svg \
  ~/.config/hypr/battery.svg

# Convert the SVG to PNG.
magick \
  -background none \
  ~/.config/hypr/battery.svg \
  ~/.config/hypr/battery.png

# Echo the resulting image path for hyprlock.
echo "$HOME"/.config/hypr/battery.png
```

```
# ~/.config/hypr/hyprlock.conf
# Change the paths as needed.
image {
    monitor =
    path = /home/chaimleib/.config/hypr/battery.png
    reload_time = 2
    reload_cmd = /home/chaimleib/.config/hypr/battery-icon.sh
    position = 0, -30
    rounding = 0
    border_size = 0
    size = 30
    halign = center
    valign = top
}
```
