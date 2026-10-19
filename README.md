# ppdeluxe

_Pronounced [Pepe Deluxé](https://en.wikipedia.org/wiki/Pepe_Delux%C3%A9)_

Bare-metal firmware for peeking and poking memory on PortalPlayer iPods.
Currently tested on iPod mini 1G, may be ported on other models with a little bit of effort (fixing UART pin mux).

**Requires some kind of accessory to access the UART interface of the dock connector. I use [HarambeJTAG](https://github.com/freemyipod/harambejtag).**

ppdeluxe is heavily based on [scythe](https://forge.voremicrocomputers.com/husky/scythe).

## Quick start tutorial

1. Install Rockbox on your iPod
2. Run `./build.sh` (requires Rust ARM toolchain + arm-none-eabi- toolchain)
3. Copy `linux.bin` from root directory into iPod storage
4. Reboot iPod (hold menu + center), once Apple logo shows up, hold play button until backlight is on
5. Set your UART monitor to 115200 bauds, 8N1 and enjoy

## Demo

```
ppdeluxe on PP5020, PROC_ID=0x55
DEV_EN=0xc28011bd DEV_EN2=0x00002000 CLCD_CLOCK_SRC=0x00000000
pp> help
  r8          r8 <addr> [count]: read count bytes from addr (hex)
  r16         r16 <addr> [count]: read count halfwords from addr (hex)
  r32         r32 <addr> [count]: read count words from addr (hex)
  w8          w8 <addr> <value>: write a byte to addr (both hex)
  w16         w16 <addr> <value>: write a halfword to addr (both hex)
  w32         w32 <addr> <value>: write a word to addr (both hex)
  help        list available commands
  history     show command history
  clear       clear the screen
pp> r32 60006038
60006038: 0000080c
pp> r32 60006038
60006038: 00005694
pp> 
```