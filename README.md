This is just a demonstration of what
[sleigh-rs](https://github.com/rbran/sleigh-rs) and
[sleigh2rust](https://github.com/rbran/sleigh2rust) are able to accomplish.

The code in each folder was generated by parsing the sleigh files from
[ghidra](https://github.com/NationalSecurityAgency/ghidra) using
[sleigh-rs](https://github.com/rbran/sleigh-rs), then generating the rust code
with [sleigh2rust](https://github.com/rbran/sleigh2rust).

This project still in a early PoC (Proof-of-Concept) stage.

The objective of this project is to implement and MVP disassembler/emulator.
Currently the code generated is unecessarelly verbose and slow, because this is
not the goal at this stage of the project.
