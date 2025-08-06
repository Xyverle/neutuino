# Neutuino
Zero dependency pure-rust simple TUI library

Supported OSes: Windows 10+, MacOS (untested), and Linux

This project is still highly work in progress and it will be a decent while until feature completeness

## Roadmap
- [x] Output (Unix)
- [x] Output (Windows)
- [x] Input (Unix) (Appears to work, more testing needed)
- [ ] Input (Windows) (WIP)
- [ ] Advanced Input (Kitty-like)
- [ ] Advanced Input (Windows)
- [ ] Events (Focus reporting, Bracketed-paste) (Unix)
- [ ] Events (Focus reporting, Bracketed-paste) (Windows)
- [ ] Mouse input (Unix)
- [ ] Mouse input (Windows)
- [ ] Feature completeness / API cleanup

## Stability
This library is unstable, for now every release should be considered breaking

## Support
This library generally attempts to have as much functionality as it can but sadly many terminal
emulators are heavily limited, there are a few protocols I have decided not to support
(i.e. [Kitty graphics protocol](https://sw.kovidgoyal.net/kitty/graphics-protocol/))

In general, this library will work best with terminals that support
[Kitty comprehensive keyboard handling](https://sw.kovidgoyal.net/kitty/keyboard-protocol/)

### Protocol Support
This is a list of terminal protocols and whether they will be supported, they still might not
work as this library is work in progress but eventually will be

Just because a protocol is listed as not planned doesn't mean it definitely won't be added, but
it is most likely not without good reason
- Standard Windows terminals (Full support planned)\*
- WinPTY (Windows psuedo-terminals) (Full support planned)
- Standard \*nix terminals (Full support planned)\*
- OSC 52 system clipboard (Full support planned)
- Kitty comprehensive keyboard handling (Full support planned)
- Kitty colored and styled underlines (Full support planned)
- Other Kitty protocols (there are a lot of them) (Not planned)

\* Do not have full support for advanced input

## Structure
This library has multiple APIs

Inside the `cli` module there are utilities for making something similar to Cargo or Git

And inside the `tui` module there are utilities for making something similar to Neovim or Emacs

And inside the rest of the library there are lower-level APIs in case you want to abstract over
this library yourself or if you need more precise control over the terminal
