Memory map at the time of stage2 startup

- `0x10000-0x4FFFF`: Kernel `.text`, `0x10000` is the x86_64 entry point from bootloader
- `0x50000-0x6FFFD`: Kernel `.data` and stack, not sure yet if this is a good idea
- `0x6FFFE`: `0x13` (magic)
- `0x6FFFF`: `0x37` (magic)
