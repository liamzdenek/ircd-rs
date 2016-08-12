lessons learned developing v1.

- I spent most of my time working on thread interop -- i can probably streamline this with macros
- I kept confusing various kinds of Strings, eg, "Nick" "Real" "Message" "Reason" etc, maybe type system can help
- define one authoritative source of all pieces of information -- I have nicks and a bunch of random crap stored everywhere, and It's easy to forget where exactly some information should be saved
- internal integers used as an offset in an array lead to overly complicated logic to derive information from that array. No idea how to improve this, but it was quite unpleasant
- longer-lived threads should be the authoritative source for any information that could be required at a point in time AFTER the worker that would intuitively be responsible for this information -- would have died (eg, the user should be responsible for its own mask, but the user thread will die upon disconnect, and that information is required to send PARTs to each channel. This could be fixed by: changing the authoritative source, changing the die condition (or possibly
timeout, but nah), or distrbuting the information before death (i've done this in part presently, and it's messy, wouldn't recommend)
