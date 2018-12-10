# Backctl

A quick and simple backlight control using `sysfs` and `udev`

## What this is

This is a super simple backlight control that adjusts the brightness value of
a backlight for me in linux using `sysfs`. It enumerates all the "backlight"
devices from `udev` and adjusts their `backlight` node accordingly.

## Why?

I discovered that xbacklight didn't work on my new laptop. Rather than fixing
the problem or using one of the many existing alternatives, I wrote this quick
utility that adjusts the backlight from userspace (after the addition of a udev
rule to set the backlight permissions accordingly). The utility is written in
rust just for fun.

