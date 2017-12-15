// Copyright 2017 Rohit Joshi <rohit.c.joshi@gmail.com>
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

#[derive(PartialEq)]
pub enum FieldCharType {
    Iso8583_n,
    Iso8583_ns,
    Iso8583_xn,
    ISO8583_a,
    Iso8583_an,
    Iso8583_ans,
    Iso8583_ansb,
    Iso8583_anp,
    Iso8583_b,
    ISO8583_z,
    Iso8583_bmp,
    Iso8583_bmps,
}
#[derive(PartialEq)]
pub enum FieldSizeType {
    Fixed,
    LlVar,
    LllVar,
    LlllVar,
    BitMap,
}

/// `IsoField` defination
pub struct IsoField {
    pub size_type: FieldSizeType,
    pub char_type: FieldCharType,
    pub length: usize,
}
/// `IsoField` implementation
impl IsoField {
    pub fn new(char_type: FieldCharType, length: usize, size_type: FieldSizeType) -> IsoField {
        IsoField {
            char_type: char_type,
            length: length,
            size_type: size_type,
        }
    }
}

/// Field Payload
#[derive(Default)]
pub struct FieldPayload {
    pub exist: bool,
    pub index: usize,
    pub len: usize,
    pub new_payload: Option<Vec<u8>>,
}

/// Field Payload impl
impl FieldPayload {
    pub fn get_new_payload_length(&self) -> usize {
        if let Some(ref m) = self.new_payload {
            return m.len();
        } else {
            return 0;
        }
    }
}

/// `IsoSpecs` Interface
/// This defines the Iso8583 message format
pub trait IsoSpecs {
    fn get_handle(&self) -> &Vec<IsoField>;
}
