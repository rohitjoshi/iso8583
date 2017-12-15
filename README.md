# ISO-8583
Iso8583 Message Parser (Serialize/Deserialize)

[![Build Status](https://travis-ci.org/rohitjoshi/iso8583.svg?branch=master)](https://travis-ci.org/rohitjoshi/iso8583)


## Benchmarking
```
test iso_msg::tests::bench_iso_msg_from_bytearray    ... bench:       1,805 ns/iter (+/- 47)
test iso_msg::tests::bench_iso_msg_to_bytearray      ... bench:       1,763 ns/iter (+/- 92)
test iso_msg::tests::bench_iso_msg_to_from_bytearray ... bench:       3,627 ns/iter (+/- 72)
```
