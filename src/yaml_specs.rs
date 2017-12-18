// Copyright 2017 Rohit Joshi <rohit.c.joshi@gmail.com>
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

use yaml_rust::yaml;
use serde_yaml;
use iso_field::FieldCharType;
use iso_field::FieldPayload;
use iso_field::IsoField;
use iso_field::FieldSizeType;
use std::collections::HashMap;
use std::collections::BTreeMap;
/// `IsoSpecs` Interface
/// This defines the Iso8583 message format
pub trait IsoSpecs {
    fn get_handle(&self) -> &Vec<IsoField>;
}

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
    pub fn new(yaml_string: &String) -> Result<YamlSpec, String> {
        let handle = try!(YamlSpec::from_string(yaml_string));
        Ok(
            YamlSpec { 
                handle: handle 
            }
        )
    }
/*
 pub label: String,
    pub char_type: FieldCharType,
    pub size_type: FieldSizeType,
    pub length: usize,
    */
    pub fn to_string(handle: &Vec<IsoField>) -> String {
        let mut btmap = BTreeMap::<usize, HashMap<String,String>>::new();
        for index in  0..handle.len() {
            let mut map = HashMap::<String,String>::with_capacity(4);
            let iso_field = &handle[index];
            map.insert(String::from("Label"), iso_field.label.clone());
            map.insert(String::from("ContentType"), String::from(iso_field.char_type.as_str()));
            map.insert(String::from("LengthType"), String::from(iso_field.size_type.as_str()));
            map.insert(String::from("Length"), iso_field.length.to_string());
            btmap.insert(index, map);
        }
        return serde_yaml::to_string(&btmap).unwrap();
    }
    
    pub fn from_string(yaml_string: &str) -> Result<Vec<IsoField>, String> {
        let fields: BTreeMap<usize, HashMap<String,String>> = match serde_yaml::from_str(&yaml_string) {
            Err(e) => {
                return Err(format!(
                                "Failed to parse yaml file. Err: {} ",e.to_string())
                );
            },
            Ok(bt) => { bt}
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
                }else if a == "ContentType" {
                    let c = FieldCharType::from_str(b);
                    if c.is_none() {
                        return Err(format!(
                                "Invalid ContentType {} for Index {}",
                                b,
                                index
                                ));
                           
                    }else {
                        char_type = c.unwrap();
                    }
                      
                }else if a == "LengthType" || a == "LenType" {

                    let lt = FieldSizeType::from_str(b);
                    if lt.is_none() {
                        return Err(format!(
                            "Invalid LengthType {} for Index {}",
                            b,
                            index
                        ));
                        
                    }else {
                        length_type = lt.unwrap();
                    }
                    
                }else if a == "Length" || a == "MaxLen" {
                    let l  = b.parse();
                    if l.is_err() {
                        return Err(format!(
                            "Invalid Length/MaxLen {} for Index {}",
                            b,
                            index
                        ));
                        
                    }else {
                        field_length : usize = l.unwrap();
                    }
                }
               
            }
             let iso_field = IsoField::new(label.as_str(), char_type, field_length as usize, length_type);
              handle.insert(*index, iso_field);
        }
        Ok(handle)
    }
}
    

#[cfg(test)]
//#[cfg(all(feature = "unstable", test))]
mod tests {
    use super::*;
    use std::{str, u32};
    use typenum::U128;

    use iso_field::FieldCharType;
    use iso_field::FieldPayload;
    use iso_field::FieldSizeType;
    use iso_field::IsoField;
    use yaml_specs::IsoSpecs;
    use std::collections::HashMap;
    use std::collections::BTreeMap;

   #[test]
   fn test_seard_yml_spec() {
       let s = 
       "
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
        
        let fields: BTreeMap::<usize, HashMap::<String,String>> = serde_yaml::from_str(&s).unwrap();
        for (key, val) in fields.iter() {
            for (a, b) in val.iter() {
                trace!("key:{}, a:{}, b:{}", key, a,b);
            }
        }
        assert_eq!(fields.len(),4)
   }
   #[test]
   fn test_yml_spec() {
       let s = 
       "
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
        assert_eq!(fields.unwrap().len(),2 );
       
        }
  
}
