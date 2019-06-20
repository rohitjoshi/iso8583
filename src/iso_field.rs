// Copyright 2017 Rohit Joshi <rohit.c.joshi@gmail.com>
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

use std::str::FromStr;

#[derive(Debug, PartialEq, Serialize, Deserialize)]
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

impl FieldCharType {
    pub fn as_str(&self) -> &'static str {
        match *self {
            FieldCharType::Iso8583_n => "n",
            FieldCharType::Iso8583_ns => "ns",
            FieldCharType::Iso8583_xn => "xs",
            FieldCharType::ISO8583_a => "a",
            FieldCharType::Iso8583_an => "an",
            FieldCharType::Iso8583_ans => "ans",
            FieldCharType::Iso8583_ansb => "ansb",
            FieldCharType::Iso8583_anp => "anp",
            FieldCharType::Iso8583_b => "b",
            FieldCharType::ISO8583_z => "z",
            FieldCharType::Iso8583_bmp => "bmp",
            FieldCharType::Iso8583_bmps => "bmps",
        }
    }
}

impl FromStr for FieldCharType {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "n" => Ok(FieldCharType::Iso8583_n),
            "ns" => Ok(FieldCharType::Iso8583_ns),
            "xs" => Ok(FieldCharType::Iso8583_xn),
            "a" => Ok(FieldCharType::ISO8583_a),
            "an" => Ok(FieldCharType::Iso8583_an),
            "ans" => Ok(FieldCharType::Iso8583_ans),
            "ansb" => Ok(FieldCharType::Iso8583_ansb),
            "anp" => Ok(FieldCharType::Iso8583_anp),
            "b" => Ok(FieldCharType::Iso8583_b),
            "z" => Ok(FieldCharType::ISO8583_z),
            "bmp" => Ok(FieldCharType::Iso8583_bmp),
            "bmps" => Ok(FieldCharType::Iso8583_bmps),
            _ => Err(()),
        }
    }
}

#[derive(Debug, PartialEq, Serialize, Deserialize)]
pub enum FieldSizeType {
    Fixed,
    LlVar,
    LllVar,
    LlllVar,
    BitMap,
}

impl FieldSizeType {
    pub fn as_str(&self) -> &'static str {
        match *self {
            FieldSizeType::Fixed => "fixed",
            FieldSizeType::LlVar => "llvar",
            FieldSizeType::LllVar => "lllvar",
            FieldSizeType::LlllVar => "llllvar",
            FieldSizeType::BitMap => "bitmap",
        }
    }
}

impl FromStr for FieldSizeType {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let s_lower = s.to_lowercase();
        match s_lower.as_str() {
            "fixed" => Ok(FieldSizeType::Fixed),
            "llvar" => Ok(FieldSizeType::LlVar),
            "lllvar" => Ok(FieldSizeType::LllVar),
            "llllvar" => Ok(FieldSizeType::LlllVar),
            "bitmap" => Ok(FieldSizeType::BitMap),
            _ => Err(()),
        }
    }
}

/// `IsoField` defination
#[derive(Debug, PartialEq, Serialize, Deserialize)]
pub struct IsoField {
    pub label: String,
    pub char_type: FieldCharType,
    pub size_type: FieldSizeType,
    pub length: usize,
}

/// `IsoField` implementation
impl IsoField {
    pub fn new(
        label: &str,
        char_type: FieldCharType,
        length: usize,
        size_type: FieldSizeType,
    ) -> IsoField {
        IsoField {
            label: String::from(label),
            char_type,
            length,
            size_type,
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
