// Copyright 2017 Rohit Joshi <rohit.c.joshi@gmail.com>
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.
use iso_field::FieldCharType;
use iso_field::FieldSizeType;
use iso_field::IsoField;
use iso_msg::IsoSpecs;
use serde_yaml;
use std::collections::BTreeMap;
use std::collections::HashMap;
use std::str::FromStr;

/// Auth spec defines the format of Iso8583 message
pub struct YamlSpec {
    handle: Vec<IsoField>,
}
///  It implements the trait defined by IsoSpecs
impl IsoSpecs for YamlSpec {
    fn get_handle(&self) -> &Vec<IsoField> {
        &self.handle
    }
}

impl YamlSpec {
    pub fn new(yaml_string: &str) -> Result<YamlSpec, String> {
        let handle = try!(YamlSpec::from_string(yaml_string));
        Ok(YamlSpec { handle })
    }
    /*
    pub label: String,
       pub char_type: FieldCharType,
       pub size_type: FieldSizeType,
       pub length: usize,
       */
    pub fn to_string(handle: &[IsoField]) -> String {
        let mut btmap = BTreeMap::<usize, HashMap<String, String>>::new();
        for (index, iso_field) in handle.iter().enumerate() {
            let mut map = HashMap::<String, String>::with_capacity(4);
            map.insert(String::from("Label"), iso_field.label.clone());
            map.insert(
                String::from("ContentType"),
                String::from(iso_field.char_type.as_str()),
            );
            map.insert(
                String::from("LengthType"),
                String::from(iso_field.size_type.as_str()),
            );
            map.insert(String::from("Length"), iso_field.length.to_string());
            btmap.insert(index, map);
        }
        serde_yaml::to_string(&btmap).unwrap()
    }

    pub fn from_string(yaml_string: &str) -> Result<Vec<IsoField>, String> {
        let fields: BTreeMap<usize, HashMap<String, String>> =
            match serde_yaml::from_str(&yaml_string) {
                Err(e) => {
                    return Err(format!(
                        "Failed to parse yaml file. Err: {} ",
                        e.to_string()
                    ));
                }
                Ok(bt) => bt,
            };
        trace!("fields.length(): {}", fields.len());
        let mut handle = Vec::<IsoField>::with_capacity(fields.len());

        for (index, val) in fields.iter() {
            let mut char_type = FieldCharType::Iso8583_ans;
            let mut length_type = FieldSizeType::Fixed;
            let mut field_length = 0;
            let mut label = String::from("");
            for (a, b) in val.iter() {
                trace!("index:{}, a:{}, b:{}", index, a, b);
                if a == "Label" {
                    label = b.to_string();
                } else if a == "ContentType" {
                    match FieldCharType::from_str(b) {
                        Err(_) => {
                            return Err(format!("Invalid ContentType {} for Index {}", b, index))
                        }
                        Ok(c) => char_type = c,
                    }
                } else if a == "LengthType" || a == "LenType" {
                    match FieldSizeType::from_str(b) {
                        Err(_) => {
                            return Err(format!("Invalid LengthType {} for Index {}", b, index))
                        }
                        Ok(lt) => length_type = lt,
                    }
                } else if a == "Length" || a == "MaxLen" {
                    let l = b.parse();
                    if l.is_err() {
                        return Err(format!("Invalid Length/MaxLen {} for Index {}", b, index));
                    } else {
                        field_length: usize = l.unwrap();
                    }
                }
            }
            let iso_field = IsoField::new(
                label.as_str(),
                char_type,
                field_length as usize,
                length_type,
            );
            handle.insert(*index, iso_field);
        }
        Ok(handle)
    }
}

#[cfg(test)]
//#[cfg(all(feature = "unstable", test))]
mod tests {
    use super::*;
    use std::collections::BTreeMap;
    use std::collections::HashMap;
    use std::str;
    use yaml_specs::IsoSpecs;

    #[test]
    fn test_seard_yml_spec() {
        let s = "
       0:
            ContentType: n
            LengthType: fixed
            Length: 4
       2:
            ContentType: an
            LengthType: llvar
            Length: 4
       1:
            ContentType: n
            LengthType: fixed
            Length: 4
       3:
            ContentType: an
            LengthType: llvar
            Length: 4
       ";

        let fields: BTreeMap<usize, HashMap<String, String>> = serde_yaml::from_str(&s).unwrap();
        for (key, val) in fields.iter() {
            for (a, b) in val.iter() {
                trace!("key:{}, a:{}, b:{}", key, a, b);
            }
        }
        assert_eq!(fields.len(), 4)
    }
    #[test]
    fn test_yml_spec() {
        let s = "
       0:
            ContentType: n
            Label : MTI
            LengthType: Fixed
            Length: 4
       1:
            ContentType: ans
            Label : test
            LengthType: LlVar
            Length: 4
       ";

        let fields = YamlSpec::from_string(s);
        assert_eq!(fields.is_ok(), true);
        assert_eq!(fields.unwrap().len(), 2);
    }

    #[test]
    fn yaml_spec_file_test() {
        use iso_msg::IsoMsg;
        use std::fs::File;
        use std::io::prelude::*;

        let file_res = File::open("spec1993.yml");
        assert_eq!(file_res.is_ok(), true);
        let mut file = file_res.unwrap();
        let mut contents = String::new();
        file.read_to_string(&mut contents).unwrap();
        let specs_res = YamlSpec::new(&contents);
        assert_eq!(specs_res.is_ok(), true);
        let handle = specs_res.unwrap();
        assert_eq!(handle.get_handle().len(), 129);

        let payload = "0100F2246481087088360000000000000004016123456717929985100300000000000013112042128251178162210581284001059006419310712815007743555555555555888Test Merchant         Richmond1    51USA011          N8402001010000000000014510002329467890120100  00054002140000000000012312340001080000000020120040001N 989";

        let iso_msg = IsoMsg::new(&handle, payload.as_bytes());
        let mut buffer = [0u8; 1024];
        {
            let res = iso_msg.get_field(0, &mut buffer);
            assert_eq!(res.unwrap(), 4);
            trace!("mti: {}", str::from_utf8(&buffer[..4]).unwrap());
            assert_eq!(&buffer[..4], "0100".as_bytes());
        }
    }

}
