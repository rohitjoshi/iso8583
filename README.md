# ISO-8583
Iso8583 Message Parser (Serialize/Deserialize)

[![Build Status](https://travis-ci.org/rohitjoshi/iso8583.svg?branch=master)](https://travis-ci.org/rohitjoshi/iso8583)


## Initial ISO-8583 Message specs from YAML File

```rust
use std::fs::File;
use std::io::prelude::*;
use iso_msg::IsoMsg;

fn main() {
    //read yaml file into a string
    let file = File::open("spec1993.yml").unwrap();
    let mut contents = String::new();
    file.read_to_string(&mut contents);
    
    let handle = YamlSpec::new(&contents).unwrap();
    assert_eq!(handle.get_handle().len(), 129);

    let payload = "0100F2246481087088360000000000000004016123456717929985100300000000000013112042128251178162210581284001059006419310712815007743555555555555888Test Merchant         Richmond1    51USA011          N8402001010000000000014510002329467890120100  00054002140000000000012312340001080000000020120040001N 989";
            
    let mut iso_msg = IsoMsg::new(&handle, payload.as_bytes());
    let mut buffer = [0u8; 1024];
    
    let res = iso_msg.get_field(0, &mut buffer);
    assert_eq!(res.unwrap(), 4);
    trace!("mti: {}", str::from_utf8(&buffer[..4]).unwrap());
    assert_eq!(&buffer[..4], "0100".as_bytes());
            
}

```


## Benchmarking
```
test iso_msg::tests::bench_iso_msg_from_bytearray    ... bench:       1,805 ns/iter (+/- 47)
test iso_msg::tests::bench_iso_msg_to_bytearray      ... bench:       1,763 ns/iter (+/- 92)
test iso_msg::tests::bench_iso_msg_to_from_bytearray ... bench:       3,627 ns/iter (+/- 72)
```
