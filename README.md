# `insert_multiple`: insert multiple items into a stream

The scenario: you have an input stream, and you know you wish to insert a number
of items into this stream at known offsets.

If your stream is a `String` or a `Vec<u8>` or similar, you shouldn't do this naively:
performance is `O(n**2)`, and you have to care about looping backwards through the
input stream to preserve your offsets.

This crate supports this use case in `O(n)`.
