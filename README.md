# buy

Simple app written by Michael Snoyman for my own usage of hledger. It provides
a shortcut for some common entries I make. Yes, this could be written as a
shell script, or a small wrapper around the `hledger` executable. I'm really
just looking for an excuse to write some more Rust.

Usage:

```
$ buy shufersal 30
```

Adds an entry to my hledger file saying we bought 30 NIS (new Israeli
shekel) worth of food at Shufersal on the appropriate credit
card. Yeah, this is super specific to my finances, no one else wants
to use this, but feel free to read the code.
