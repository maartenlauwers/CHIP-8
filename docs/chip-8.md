# CHIP-8 Architecture Notes

## Terminology

### Emulation vs Simulation
### Emulation

Focuses on replicating the behavior (of the hardware system) so
software designed for it can run unmodified. The interface of the original 
hardware must be respected, while the implementation can be different.

This is what this CHIP-8 project focuses on.

### Simulation

Focuses on modeling the precise behavior of the original system. Both interface 
and implementation are important.

## CHIP-8

A virtual machine and interpreted language.

It is made up of the following components:

- Memory
- Registers
- Stack
- Timers
- Input
- Display
- Instruction Set

### CPU

# Sources

- hexdump (https://www.youtube.com/watch?v=7HVXBzsujyc)
- https://aquova.net/emudev/chip8/1-intro.html
- http://devernay.free.fr/hacks/chip8/C8TECH10.HTM#2.5