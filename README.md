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
