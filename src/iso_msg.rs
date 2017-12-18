// Copyright 2017 Rohit Joshi <rohit.c.joshi@gmail.com>
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.


use std::str;
use bit_array::BitArray;
use typenum::U128;
use std::borrow::Cow;
use iso_field::FieldCharType;
use iso_field::FieldPayload;
use iso_field::IsoField;
use iso_field::FieldSizeType;



/// `IsoSpecs` Interface
/// This defines the Iso8583 message format
pub trait IsoSpecs {
    fn get_handle(&self) -> &Vec<IsoField>;
}

/// `IsoMsg`
pub struct IsoMsg<'a, 'b> {
    payload: Cow<'a, [u8]>,
    iso_spec: &'b IsoSpecs,
    fields: Vec<FieldPayload>,
}

impl<'a, 'b> IsoMsg<'a, 'b> {
    pub fn new(iso_spec: &'b IsoSpecs, payload: &'a [u8]) -> IsoMsg<'a, 'b> {
        let mut fields = Vec::with_capacity(iso_spec.get_handle().len());

        IsoMsg::from_byte_array(iso_spec, &mut fields, payload);

        IsoMsg {
            iso_spec: iso_spec,
            payload: Cow::Borrowed(payload),

            //bitmap : BitArray::<u8, U128>::from_elem(false),
            fields: fields,
        }
    }

    pub fn remove_field(&mut self, index: usize) -> Result<(), &str> {
        assert!(index < self.fields.len());
        assert!(index < self.iso_spec.get_handle().len());
        self.fields[index].exist = false;
        Ok(())
    }


    pub fn set_field(&mut self, index: usize, buffer: &[u8]) -> Result<(), &str> {
        trace!(
            "set_field: index:{}, buffer:{}",
            index,
            str::from_utf8(&buffer).unwrap()
        );
        assert!(index < self.fields.len());
        assert!(index < self.iso_spec.get_handle().len());
        assert!(buffer.len() <= self.iso_spec.get_handle()[index].length);

        let len_prefix = self.get_field_length_prefix(index);
        let total_lenth = buffer.len() + len_prefix;
        let mut v = Vec::with_capacity(total_lenth);
        trace!(
            "buffer.len():{}, iso_spec.get_handle()[index].length:{}",
            buffer.len(),
            self.iso_spec.get_handle()[index].length
        );
        if len_prefix > 0 {

            v.extend_from_slice(format!("{:0w$}", buffer.len(), w = len_prefix).as_bytes());
        }
        v.extend_from_slice(buffer);

        trace!(
            "index:{}, set_extend_from_slice : v {}",
            index,
            str::from_utf8(&v).unwrap()
        );
        trace!("set_field: v.len:{}", v.len());
        self.fields[index].new_payload = Some(v);
        self.fields[index].exist = true;
        Ok(())
    }

    pub fn get_field_length_prefix(&self, index: usize) -> usize {
        match self.iso_spec.get_handle()[index].size_type {
            FieldSizeType::LlVar | FieldSizeType::LllVar | FieldSizeType::LlllVar => 3,
            _ => 0,
        }
    }

    pub fn get_field(&self, index: usize, buffer: &mut [u8]) -> Result<usize, &str> {

        let res = self.get_field_raw(index, buffer);
        if res.is_err() {
            return Err(res.err().unwrap());
        }

        let (len, field_len_prefix) = res.unwrap();
        if field_len_prefix > 0 {
            let temp_buff = buffer[field_len_prefix..len].to_vec();
            buffer[0..len - field_len_prefix].copy_from_slice(&temp_buff[..]);
        }
        Ok(len - field_len_prefix)
    }

    fn get_field_raw(&self, index: usize, buffer: &mut [u8]) -> Result<(usize, usize), &str> {
        assert!(index < self.fields.len());
        let field = &self.fields[index];
        if !field.exist {
            return Err("Field not set");
        }

        if field.new_payload.is_some() {
            trace!("new_payload exist");
            if let Some(ref m) = field.new_payload {
                if buffer.len() >= m.len() {
                    let len_prefix = self.get_field_length_prefix(index);
                    buffer[..m.len()].copy_from_slice(&m[..m.len()]);
                    return Ok((m.len(), len_prefix));
                }
            }
            return Err("Input buffer is smaller than field value");

        }
        if field.len == 0 {
            return Err("Field not set");
        }
        if buffer.len() >= field.len && self.payload.len() >= (field.len + field.index) {
            let len_prefix = self.get_field_length_prefix(index);
            buffer[0..field.len].copy_from_slice(
                &self.payload[field.index..
                                  field.index +
                                      field.len],
            );
            Ok((field.len, len_prefix))
        } else {
            Err("Input buffer is smaller than field value")
        }


    }



    pub fn is_bit_set(input: u32, n: u8) -> bool {
        if n < 32 { input & (1 << n) != 0 } else { false }
    }

    pub fn process_bitmap(
        iso_spec: &IsoSpecs,
        index: usize,
        bitmap_bytes: &[u8],
    ) -> (Vec<BitArray<u64, U128>>, usize) {

        let bitmap_length = iso_spec.get_handle()[index].length;
        let bitmap_size = bitmap_length * 8;
        let num_iteration: usize = (iso_spec.get_handle().len() - index + 63) / bitmap_size;
        let i = 0;

        let mut bit_map_index = index;
        let num_bits = 32; //note: we are using rev() of range e.g (0..num_bits).rev() so 32 would start with 31
        let mut bit_arrays = Vec::<BitArray<u64, U128>>::with_capacity(num_iteration);
        for _ in 0..num_iteration {
            bit_arrays.push(BitArray::<u64, U128>::from_elem(false));
        }

        let mut bit_arrays_index = 0;

        //let bitmap = unsafe { str::from_utf8_unchecked(bitmap_bytes) };
        let bitmap = str::from_utf8(bitmap_bytes).unwrap();
        trace!("index:{}, bitmap:{}", index, bitmap);
        let mut field_index = 0; //current field index
        while i < num_iteration {

            //move to the next field
            if i == 0 {
                field_index += 1; // skip field index 0. This is to check if bit 31 is set for 2ndory bitmap
            }
            let mut ms_num_bit = num_bits;
            if i == 0 {
                ms_num_bit -= 1;
            }
            let ms = u32::from_str_radix(&bitmap[bit_map_index..bit_map_index + 8], 16).unwrap();

            //  for x in (num_bits..0).rev() {
            for x in (0..ms_num_bit).rev() {
                let bit_set = IsoMsg::is_bit_set(ms, x as u8);
                trace!("ms: bit {} field_index:{} is {}", x, field_index, bit_set);
                bit_arrays[bit_arrays_index].set(field_index, bit_set);
                field_index += 1;
            }
            trace!("ms: {}", ms);
            bit_map_index += 8;

            let ls = u32::from_str_radix(&bitmap[bit_map_index..bit_map_index + 8], 16).unwrap();
            for x in (0..num_bits).rev() {
                let bit_set = IsoMsg::is_bit_set(ls, x as u8);
                trace!("ms: bit {} field_index:{} is {}", x, field_index, bit_set);
                bit_arrays[bit_arrays_index].set(field_index, bit_set);
                field_index += 1;
            }
            bit_map_index += 8;
            trace!("ls: {}", ls);


            bit_arrays_index = field_index / bitmap_size;
            if i == 0 && !IsoMsg::is_bit_set(ms, 31u8) {
                // if 2ndory bitmap doesn't exist, comeout
                trace!("2ndory bitmap doesn't exis");
                break;
            }
        }


        (bit_arrays, bit_map_index + index - 8)

    }

    pub fn convert_u32_be(array: &[u8]) -> u32 {
        assert_eq!(array.len(), 4);
        (u32::from(array[0]) << 24) + (u32::from(array[1]) << 16) + (u32::from(array[2]) << 8) +
            (u32::from(array[3]) << 0)
    }

    pub fn convert_u32_le(array: &[u8]) -> u32 {
        assert_eq!(array.len(), 4);
        (u32::from(array[0]) << 0) + (u32::from(array[1]) << 8) + (u32::from(array[2]) << 16) +
            (u32::from(array[3]) << 24)
    }

    pub fn to_byte_array(&self, buffer: &mut [u8]) -> usize {
        let mut buffer_index = 0usize;
        let num_iteration: usize = (self.iso_spec.get_handle().len() - 1 + 63) / 128;
        let mut bit_arrays = Vec::<BitArray<u64, U128>>::with_capacity(num_iteration);
        for _ in 0..num_iteration {
            bit_arrays.push(BitArray::<u64, U128>::from_elem(false));
        }
        let mut bit_array_index = 0;
        let mut bit_index = 0;
        let mut bitmap_field_index = 0;

        let mut bitmap_found = false;

        for index in 0..self.fields.len() {
            bit_array_index = index / 128;

            if !bitmap_found &&
                (self.iso_spec.get_handle()[index].char_type == FieldCharType::Iso8583_bmp ||
                     self.iso_spec.get_handle()[index].char_type == FieldCharType::Iso8583_bmps)
            {
                bitmap_field_index = index;
                bitmap_found = true;
                bit_index = buffer_index;
                let res = self.get_field_raw(index, &mut buffer[buffer_index..]);
                if res.is_ok() {
                    let (field_total_len, _) = res.unwrap();
                    buffer_index += field_total_len;
                }
            } else {

                let res = self.get_field_raw(index, &mut buffer[buffer_index..]);
                if res.is_ok() {
                    if bitmap_found {
                        bit_arrays[bit_array_index].set(index - bitmap_field_index, true);
                        trace!(
                            "index:{}, buffer[buffer_index..]:{}",
                            index,
                            str::from_utf8(&buffer[buffer_index..]).unwrap()
                        );
                    }
                    let (field_total_len, _) = res.unwrap();
                    buffer_index += field_total_len;
                }
            }

        }
        //override bitmap
        let mut bitmap = String::with_capacity(bit_array_index * 16);
        for (i, bit_array_item) in bit_arrays.iter_mut().enumerate().take(bit_array_index) {
            //for i in 0..bit_array_index {
            if i == 0 && bit_array_item.len() > 64 {
                bit_array_item.set(0, true);
            }
            let bytes = bit_array_item.to_bytes();
            let mut byte_index = 0;

            while byte_index < bytes.len() {
                let ms_str = IsoMsg::convert_u32_be(&bytes[byte_index..byte_index + 4]);
                byte_index += 4;
                bitmap.push_str(&format!("{:08X}", ms_str));

            }
        }
        buffer[bit_index..bitmap.len() + bit_index]
            .copy_from_slice(&bitmap.as_bytes()[0..bitmap.len()]);
        buffer_index
    }

    pub fn get_field_length(iso_field: &IsoField, input_buffer: &[u8]) -> usize {
        match iso_field.size_type {
            FieldSizeType::Fixed => iso_field.length,
            FieldSizeType::LlVar | FieldSizeType::LllVar | FieldSizeType::LlllVar => {
                let str_digits = unsafe { str::from_utf8_unchecked(&input_buffer[0..3]) };
                usize::from_str_radix(str_digits, 10).unwrap() + 3
            }
            _ => 0,
        }
    }

    pub fn from_byte_array(
        iso_spec: &IsoSpecs,
        fields: &mut Vec<FieldPayload>,
        input_buffer: &[u8],
    ) {
        let mut payload_index = 0usize;
        let mut found_bitmap = false;
        let mut bitmap_field_index = 0;
        let mut bit_arrays = Vec::<BitArray<u64, U128>>::with_capacity(0);
        for i in 0..iso_spec.get_handle().len() {
            let iso_field: &IsoField = &iso_spec.get_handle()[i];

            let mut field = FieldPayload::default();
            if !found_bitmap &&
                (iso_field.char_type == FieldCharType::Iso8583_bmp ||
                     iso_field.char_type == FieldCharType::Iso8583_bmps)
            {
                found_bitmap = true;
                field.index = payload_index;

                field.exist = true;
                bitmap_field_index = i;

                let (bitarrays, len) = IsoMsg::process_bitmap(iso_spec, field.index, input_buffer);
                field.len = len;
                bit_arrays = bitarrays;
                payload_index += field.len; //(iso_field.length * len/16);
                trace!(
                    "iso_field.length:{}, field.index:{}, payload_index:{}, bitmap: {}",
                    iso_field.length,
                    field.index,
                    payload_index,
                    str::from_utf8(&input_buffer[field.index..len + field.index]).unwrap()
                );

                trace!("bit_arrays:{}", bit_arrays.len());

            } else {
                let mut field_exist = true; //until bitmap found, assume field exist
                if found_bitmap {
                    if bit_arrays[0].get(i - bitmap_field_index).unwrap() {
                        field_exist = true;
                        trace!("Field {} exists.", i);
                    } else {
                        field_exist = false;
                    }
                }

                if field_exist {
                    field.index = payload_index;
                    field.len = IsoMsg::get_field_length(iso_field, &input_buffer[payload_index..]);
                    field.exist = true;
                    payload_index += field.len;
                    trace!(
                        "iso_field.length:{}, field.index:{}, payload_index:{}, ",
                        iso_field.length,
                        field.index,
                        payload_index
                    );
                }
            }

            fields.push(field)


        }
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

    use yaml_specs::YamlSpec;

    /// Auth spec defines the format of Iso8583 message
    pub struct AuthSpecs {
        handle: Vec<IsoField>,
    }
    impl AuthSpecs {
        pub fn new() -> AuthSpecs {
            AuthSpecs { handle: Util::define_auth_specs() }
        }
    }

    ///  It implements the trait defined by IsoSpecs
    impl IsoSpecs for AuthSpecs {
        fn get_handle(&self) -> &Vec<IsoField> {
            &self.handle
        }
    }


    struct Util;

    impl Util {
        pub fn define_auth_specs() -> Vec<IsoField> {
            let h = vec![
IsoField::new("Message Type Indicator",FieldCharType::Iso8583_ns ,  4,FieldSizeType::Fixed), // Message Type Indicator
IsoField::new("Bitmap",FieldCharType::Iso8583_bmps, 16,FieldSizeType::BitMap), // Bitmap
IsoField::new("Primary Account Number",FieldCharType::Iso8583_ns , 19,FieldSizeType::LlVar), // Primary Account Number
IsoField::new("Processing Code",FieldCharType::Iso8583_ns ,  6,FieldSizeType::Fixed), // Processing Code
IsoField::new("Amount, Txn",FieldCharType::Iso8583_ns , 12,FieldSizeType::Fixed), // Amount, Txn
IsoField::new("Amount, Reconciliation",FieldCharType::Iso8583_ns , 12,FieldSizeType::Fixed), // Amount, Reconciliation
IsoField::new("Amount, Cardholder Billing",FieldCharType::Iso8583_ns , 12,FieldSizeType::Fixed), // Amount, Cardholder Billing
IsoField::new("Date and Time, Transmission",FieldCharType::Iso8583_ns , 10,FieldSizeType::Fixed), // Date and Time, Transmission
IsoField::new("Amount, Cardholder Billing Fee",FieldCharType::Iso8583_ns ,  8,FieldSizeType::Fixed), // Amount, Cardholder Billing Fee
IsoField::new("Conversion Rate, Reconciliation",FieldCharType::Iso8583_ns ,  8,FieldSizeType::Fixed), // Conversion Rate, Reconciliation
IsoField::new("Conversion Rate, Cardholder Billing",FieldCharType::Iso8583_ns ,  8,FieldSizeType::Fixed), // Conversion Rate, Cardholder Billing
IsoField::new("Systems Trace Audit Number",FieldCharType::Iso8583_ns ,  6,FieldSizeType::Fixed), // Systems Trace Audit Number
IsoField::new("Date and Time, Local Txn",FieldCharType::Iso8583_ns ,  6,FieldSizeType::Fixed), // Date and Time, Local Txn
IsoField::new("Date, Effective",FieldCharType::Iso8583_ns ,  4,FieldSizeType::Fixed), // Date, Effective
IsoField::new("Date, Expiration",FieldCharType::Iso8583_ns ,  4,FieldSizeType::Fixed), // Date, Expiration
IsoField::new("Date, Settlement",FieldCharType::Iso8583_ns ,  4,FieldSizeType::Fixed), // Date, Settlement
IsoField::new("Date, Conversion",FieldCharType::Iso8583_ns ,  4,FieldSizeType::Fixed), // Date, Conversion
IsoField::new("Date, Capture",FieldCharType::Iso8583_ns ,  4,FieldSizeType::Fixed), // Date, Capture
IsoField::new("Merchant Type",FieldCharType::Iso8583_ns ,  4,FieldSizeType::Fixed), // Merchant Type
IsoField::new("Country Code, Acquiring Inst",FieldCharType::Iso8583_ns ,  3,FieldSizeType::Fixed), // Country Code, Acquiring Inst
IsoField::new("Country Code, Primary Account Number",FieldCharType::Iso8583_ns ,  3,FieldSizeType::Fixed), // Country Code, Primary Account Number
IsoField::new("Country Code, Forwarding Inst",FieldCharType::Iso8583_ns ,  3,FieldSizeType::Fixed), // Country Code, Forwarding Inst
IsoField::new("Point of Service Data Code",FieldCharType::Iso8583_ns ,  3,FieldSizeType::Fixed), // Point of Service Data Code
IsoField::new("Card Sequence Number",FieldCharType::Iso8583_ns ,  3,FieldSizeType::Fixed), // Card Sequence Number
IsoField::new("Function Code",FieldCharType::Iso8583_ns ,  3,FieldSizeType::Fixed), // Function Code
IsoField::new("Message Reason Code",FieldCharType::Iso8583_ns ,  2,FieldSizeType::Fixed), // Message Reason Code
IsoField::new("Card Acceptor Business Code",FieldCharType::Iso8583_ns ,  2,FieldSizeType::Fixed), // Card Acceptor Business Code
IsoField::new("Approval Code Length",FieldCharType::Iso8583_ns ,  1,FieldSizeType::Fixed), // Approval Code Length
IsoField::new("Date, Reconciliation",FieldCharType::Iso8583_ns ,  9,FieldSizeType::Fixed), // Date, Reconciliation
IsoField::new("Reconciliation Indicator",FieldCharType::Iso8583_ns ,  9,FieldSizeType::Fixed), // Reconciliation Indicator
IsoField::new("Amounts, Original",FieldCharType::Iso8583_ns , 24,FieldSizeType::Fixed), // Amounts, Original
IsoField::new("Acquirer Reference Data",FieldCharType::Iso8583_ans, 99,FieldSizeType::LlVar), // Acquirer Reference Data
IsoField::new(" Acquirer Inst Id Code",FieldCharType::Iso8583_ns , 11,FieldSizeType::LlVar), // Acquirer Inst Id Code
IsoField::new("Forwarding Inst Id Code",FieldCharType::Iso8583_ns , 11,FieldSizeType::LlVar), // Forwarding Inst Id Code
IsoField::new("Primary Account Number, Extended",FieldCharType::Iso8583_ns , 28,FieldSizeType::LlVar), // Primary Account Number, Extended
IsoField::new("Track 2 Data",FieldCharType::ISO8583_z  , 37,FieldSizeType::LlVar), // Track 2 Data
IsoField::new("Track 3 Data",FieldCharType::ISO8583_z  ,104,FieldSizeType::LllVar), // Track 3 Data
IsoField::new("Retrieval Reference Number",FieldCharType::Iso8583_anp, 12,FieldSizeType::Fixed), // Retrieval Reference Number
IsoField::new("Approval Code",FieldCharType::Iso8583_anp,  6,FieldSizeType::Fixed), // Approval Code
IsoField::new("Action Code",FieldCharType::Iso8583_ns ,  2,FieldSizeType::Fixed), // Action Code
IsoField::new("Service Code",FieldCharType::Iso8583_ns ,  3,FieldSizeType::Fixed), // Service Code
IsoField::new("Card Acceptor Terminal Id",FieldCharType::Iso8583_ans,  8,FieldSizeType::Fixed), // Card Acceptor Terminal Id
IsoField::new("Card Acceptor Id Code",FieldCharType::Iso8583_ans, 15,FieldSizeType::Fixed), // Card Acceptor Id Code
IsoField::new("Card Acceptor Name/Location",FieldCharType::Iso8583_ans, 40,FieldSizeType::Fixed), // Card Acceptor Name/Location
IsoField::new("dditional Response Data",FieldCharType::Iso8583_ans, 99,FieldSizeType::LlVar), // Additional Response Data
IsoField::new("Track 1 Data",FieldCharType::Iso8583_ans, 76,FieldSizeType::LlVar), // Track 1 Data
IsoField::new("Amounts, Fees",FieldCharType::Iso8583_ans,204,FieldSizeType::LllVar), // Amounts, Fees
IsoField::new("Additional Data - National",FieldCharType::Iso8583_ans,999,FieldSizeType::LllVar), // Additional Data - National
IsoField::new("Additional Data - Private",FieldCharType::Iso8583_ans,999,FieldSizeType::LllVar), // Additional Data - Private
IsoField::new("Currency Code, Txn",FieldCharType::Iso8583_an ,  3,FieldSizeType::Fixed), // Currency Code, Txn
IsoField::new("Currency Code, Reconciliation",FieldCharType::Iso8583_an ,  3,FieldSizeType::Fixed), // Currency Code, Reconciliation
IsoField::new("Currency Code, Cardholder Billing",FieldCharType::Iso8583_an ,  3,FieldSizeType::Fixed), // Currency Code, Cardholder Billing
IsoField::new("Personal Id Number (PIN) Data",FieldCharType::Iso8583_ans  ,  16,FieldSizeType::Fixed), // Personal Id Number (PIN) Data
IsoField::new("Security Related Control Information",FieldCharType::Iso8583_ns  , 16,FieldSizeType::Fixed), // Security Related Control Information
IsoField::new("Amounts, Additional",FieldCharType::Iso8583_ans,120,FieldSizeType::LllVar), // Amounts, Additional
IsoField::new("IC Card System Related Data",FieldCharType::Iso8583_ans  ,999,FieldSizeType::LllVar), // IC Card System Related Data
IsoField::new("Original Data Elements",FieldCharType::Iso8583_ans , 35,FieldSizeType::LlVar), // Original Data Elements
IsoField::new("Authorization Life Cycle Code",FieldCharType::Iso8583_ans ,999,FieldSizeType::LllVar), // Authorization Life Cycle Code
IsoField::new("Authorizing Agent Inst Id Cod",FieldCharType::Iso8583_ans ,999,FieldSizeType::LllVar), // Authorizing Agent Inst Id Code
IsoField::new("Transport Data",FieldCharType::Iso8583_ans,999,FieldSizeType::LllVar), // Transport Data
IsoField::new("Reserved for National use",FieldCharType::Iso8583_ans,999,FieldSizeType::LllVar), // Reserved for National use
IsoField::new("Reserved for National use",FieldCharType::Iso8583_ans,999,FieldSizeType::LllVar), // Reserved for National use
IsoField::new("Reserved for Private use",FieldCharType::Iso8583_ans,999,FieldSizeType::LllVar), // Reserved for Private use
IsoField::new("Reserved for Private use",FieldCharType::Iso8583_ans,999,FieldSizeType::LllVar), // Reserved for Private use
IsoField::new("Message Authentication Code Field",FieldCharType::Iso8583_b  ,  8,FieldSizeType::Fixed), // Message Authentication Code Field
IsoField::new("Reserved for ISO use",FieldCharType::Iso8583_b  ,  8,FieldSizeType::Fixed), // Reserved for ISO use
IsoField::new("Reconciliation code , Original Fees",FieldCharType::Iso8583_ans,  1,FieldSizeType::Fixed), //Reconciliation code , Original Fees
IsoField::new("Extended Payment Data",FieldCharType::Iso8583_ns ,  2,FieldSizeType::Fixed), // Extended Payment Data
IsoField::new("Country Code, Receiving Inst",FieldCharType::Iso8583_ns ,  3,FieldSizeType::Fixed), // Country Code, Receiving Inst
IsoField::new("Country Code, Settlement Inst",FieldCharType::Iso8583_ns ,  3,FieldSizeType::Fixed), // Country Code, Settlement Inst
IsoField::new("Network Management Information Code",FieldCharType::Iso8583_ns ,  3,FieldSizeType::Fixed), // Network Management Information Code
IsoField::new("Message Number",FieldCharType::Iso8583_ns ,  6,FieldSizeType::Fixed), // Message Number
IsoField::new("Data Record",FieldCharType::Iso8583_ans,999,FieldSizeType::LllVar), // Data Record
IsoField::new("Date, Action",FieldCharType::Iso8583_ns ,  6,FieldSizeType::Fixed), // Date, Action
IsoField::new("Credits, Number",FieldCharType::Iso8583_ns , 10,FieldSizeType::Fixed), // Credits, Number
IsoField::new("Credits, Reversal Number",FieldCharType::Iso8583_ns , 10,FieldSizeType::Fixed), // Credits, Reversal Number
IsoField::new("Debits, Number",FieldCharType::Iso8583_ns , 10,FieldSizeType::Fixed), // Debits, Number
IsoField::new("Debits, Reversal Number",FieldCharType::Iso8583_ns , 10,FieldSizeType::Fixed), // Debits, Reversal Number
IsoField::new("Transfer, Number",FieldCharType::Iso8583_ns , 10,FieldSizeType::Fixed), // Transfer, Number
IsoField::new("Transfer, Reversal Number",FieldCharType::Iso8583_ns , 10,FieldSizeType::Fixed), // Transfer, Reversal Number
IsoField::new("Inquiries, Number",FieldCharType::Iso8583_ns , 10,FieldSizeType::Fixed), // Inquiries, Number
IsoField::new("Authorizations, Number",FieldCharType::Iso8583_ns , 10,FieldSizeType::Fixed), // Authorizations, Number
IsoField::new("Inquiries, Reversal Number",FieldCharType::Iso8583_ns , 10,FieldSizeType::Fixed), // Inquiries, Reversal Number
IsoField::new("Payments, Number",FieldCharType::Iso8583_ns , 10,FieldSizeType::Fixed), // Payments, Number
IsoField::new("Payments, Reversal Number",FieldCharType::Iso8583_ns , 10,FieldSizeType::Fixed), // Payments, Reversal Number
IsoField::new("Fee Collections, Number",FieldCharType::Iso8583_ns , 10,FieldSizeType::Fixed), // Fee Collections, Number
IsoField::new("Credits, Amount",FieldCharType::Iso8583_ns , 16,FieldSizeType::Fixed), // Credits, Amount
IsoField::new("Credits, Reversal Amount",FieldCharType::Iso8583_ns , 16,FieldSizeType::Fixed), // Credits, Reversal Amount
IsoField::new("Debits, Amount",FieldCharType::Iso8583_ns , 16,FieldSizeType::Fixed), // Debits, Amount
IsoField::new("Debits, Reversal Amount",FieldCharType::Iso8583_ns , 16,FieldSizeType::Fixed), // Debits, Reversal Amount
IsoField::new("Authorizations, Reversal Number",FieldCharType::Iso8583_ns , 42,FieldSizeType::Fixed), // Authorizations, Reversal Number
IsoField::new("Country Code, Txn Destination Inst",FieldCharType::Iso8583_ns ,  3,FieldSizeType::Fixed), // Country Code, Txn Destination Inst
IsoField::new("Country Code, Txn Originator Inst",FieldCharType::Iso8583_ns ,  3,FieldSizeType::Fixed), // Country Code, Txn Originator Inst
IsoField::new("Txn Destination Inst Id Code",FieldCharType::Iso8583_ns , 11,FieldSizeType::LlVar), // Txn Destination Inst Id Code
IsoField::new("Txn Originator Inst Id Code",FieldCharType::Iso8583_ns , 11,FieldSizeType::LlVar), // Txn Originator Inst Id Code
IsoField::new("Card Issuer Reference Data",FieldCharType::Iso8583_ans, 42,FieldSizeType::Fixed), // Card Issuer Reference Data
IsoField::new("Key Management Data",FieldCharType::Iso8583_b  ,999,FieldSizeType::LllVar), // Key Management Data
IsoField::new("Amount, Net Reconciliation",FieldCharType::Iso8583_xn , 17,FieldSizeType::Fixed), // Amount, Net Reconciliation
IsoField::new("Payee",FieldCharType::Iso8583_ans, 25,FieldSizeType::Fixed), // Payee
IsoField::new("Settlement Inst Id Code",FieldCharType::Iso8583_an , 11,FieldSizeType::LlVar), // Settlement Inst Id Code
IsoField::new("Receiving Inst Id Code",FieldCharType::Iso8583_ns , 11,FieldSizeType::LlVar), // Receiving Inst Id Code
IsoField::new("File Name",FieldCharType::Iso8583_ans, 17,FieldSizeType::LlVar), // File Name
IsoField::new("Account Id 1",FieldCharType::Iso8583_ans, 28,FieldSizeType::LlVar), // Account Id 1
IsoField::new("Account Id 2",FieldCharType::Iso8583_ans, 28,FieldSizeType::LlVar), // Account Id 2
IsoField::new("Txn Description",FieldCharType::Iso8583_ans,255,FieldSizeType::LllVar), // Txn Description
IsoField::new("Credits, Chargeback Amount",FieldCharType::Iso8583_ns , 16,FieldSizeType::Fixed), // Credits, Chargeback Amount
IsoField::new("Debits, Chargeback Amount",FieldCharType::Iso8583_ns , 16,FieldSizeType::Fixed), // Debits, Chargeback Amount
IsoField::new("Credits, Chargeback Number",FieldCharType::Iso8583_ns , 10,FieldSizeType::Fixed), // Credits, Chargeback Number
IsoField::new("Debits, Chargeback Number",FieldCharType::Iso8583_ns , 10,FieldSizeType::Fixed), // Debits, Chargeback Number
IsoField::new("Credits, Fee Amounts",FieldCharType::Iso8583_ans, 84,FieldSizeType::LlVar), // Credits, Fee Amounts
IsoField::new("Debits, Fee Amounts",FieldCharType::Iso8583_ans, 84,FieldSizeType::LlVar), // Debits, Fee Amounts
IsoField::new("Reserved for ISO use",FieldCharType::Iso8583_ns,12,FieldSizeType::Fixed ), // Reserved for ISO use
IsoField::new("Reserved for ISO use",FieldCharType::Iso8583_ans,999,FieldSizeType::LllVar), // Reserved for ISO use
IsoField::new("Reserved for ISO use",FieldCharType::Iso8583_ans,999,FieldSizeType::LllVar), // Reserved for ISO use
IsoField::new("Reserved for ISO use",FieldCharType::Iso8583_ans,999,FieldSizeType::LllVar), // Reserved for ISO use
IsoField::new("Reserved for ISO use",FieldCharType::Iso8583_ans,999,FieldSizeType::LllVar), // Reserved for ISO use
IsoField::new("Reserved for National use",FieldCharType::Iso8583_ans,999,FieldSizeType::LllVar), // Reserved for National use
IsoField::new("Reserved for National use",FieldCharType::Iso8583_ans,999,FieldSizeType::LllVar), // Reserved for National use
IsoField::new("Reserved for National use",FieldCharType::Iso8583_ans,999,FieldSizeType::LllVar), // Reserved for National use
IsoField::new("Reserved for National use",FieldCharType::Iso8583_ans,999,FieldSizeType::LllVar), // Reserved for National use
IsoField::new("Reserved for National use",FieldCharType::Iso8583_ans,999,FieldSizeType::LllVar), // Reserved for National use
IsoField::new("Reserved for National use",FieldCharType::Iso8583_ans,999,FieldSizeType::LllVar), // Reserved for National use
IsoField::new("Reserved for National use",FieldCharType::Iso8583_ans,999,FieldSizeType::LllVar), // Reserved for National use
IsoField::new("Reserved for Private use",FieldCharType::Iso8583_ans,999,FieldSizeType::LllVar), // Reserved for Private use
IsoField::new("Reserved for Private use",FieldCharType::Iso8583_ans,999,FieldSizeType::LllVar), // Reserved for Private use
IsoField::new("Reserved for Private use",FieldCharType::Iso8583_ans,999,FieldSizeType::LllVar), // Reserved for Private use
IsoField::new("Reserved for Private use",FieldCharType::Iso8583_ans,999,FieldSizeType::LllVar), // Reserved for Private use
IsoField::new("Reserved for Private use",FieldCharType::Iso8583_ans,999,FieldSizeType::LllVar), // Reserved for Private use
IsoField::new("Message Authentication Code Field",FieldCharType::Iso8583_b  ,  8,FieldSizeType::Fixed),  // Message Authentication Code Field
  ];
            return h;
        }
    }


    // use std::collections::BitSet;
    fn is_bit_set(input: u32, n: u8) -> bool {
        if n < 32 { input & (1 << n) != 0 } else { false }
    }

    #[test]
    fn bit_array_test() {
        let bitmap = "F2246481087088360000000000000004";
        let num_bytes: usize = bitmap.len() / 16;
        //let mut bs = BitSet::with_capacity(128);
        let mut bv = BitArray::<u64, U128>::from_elem(false);

        let mut field_index = 0;
        let mut bit_map_index = 0;
        let mut num_bits;
        for i in 0..num_bytes {
            //move to the next field
            if i == 0 {
                field_index += 1;
            }
            if i == 0 {
                num_bits = 30;
            } else {
                num_bits = 31;
            }
            let mut ms = u32::from_str_radix(&bitmap[bit_map_index..bit_map_index + 8], 16)
                .unwrap();

            //  for x in (num_bits..0).rev() {
            for x in (0..num_bits).rev() {
                //for x in (num_bits..0).step_by(-1) {
                bv.set(field_index, is_bit_set(ms, x as u8));
                field_index += 1;
            }
            bit_map_index += 8;
            let mut ls = u32::from_str_radix(&bitmap[bit_map_index..bit_map_index + 8], 16)
                .unwrap();

            for x in (0..num_bits).rev() {
                //for x in (num_bits..0).step_by(-1) {
                bv.set(field_index, is_bit_set(ls, x as u8));
                field_index += 1;
            }
            if i == 0 && !is_bit_set(ms, 31u8) {
                break;
            }

        }


    }

    #[test]
    fn process_bitmap_test() {
        let bitmap = "0010F2246481087088360000000000000004";
        let handle = AuthSpecs::new();
        let (bit_arrays, _) = IsoMsg::process_bitmap(&handle, 1, bitmap.as_bytes());
        assert_eq!(bit_arrays.len(), 1);
        // let mut field_index =2;
       /*
        for (i, item) in bit_arrays[0].iter().enumerate() {
            if (item) {
                print!("{}, ", i+2 );
            }
        }*/
    }

    #[test]
    fn from_byte_array_test() {
        let payload = "0100F2246481087088360000000000000004016123456717929985100300000000000013112042128251178162210581284001059006419310712815007743555555555555888Test Merchant         Richmond1    51USA011          N8402001010000000000014510002329467890120100  00054002140000000000012312340001080000000020120040001N 989";

        let iso_spec = AuthSpecs::new();
        trace!(
            "iso_spec.get_handle().len(): {}",
            iso_spec.get_handle().len()
        );
        let mut fields = Vec::<FieldPayload>::with_capacity(iso_spec.get_handle().len());

        trace!("Fields length:{}", fields.len());

        IsoMsg::from_byte_array(&iso_spec, &mut fields, payload.as_bytes());

    }

    #[test]
    fn init_iso_msg_test() {
        let payload = "0100F2246481087088360000000000000004016123456717929985100300000000000013112042128251178162210581284001059006419310712815007743555555555555888Test Merchant         Richmond1    51USA011          N8402001010000000000014510002329467890120100  00054002140000000000012312340001080000000020120040001N 989";
        let handle = AuthSpecs::new();
        let mut iso_msg = IsoMsg::new(&handle, payload.as_bytes());
        let mut buffer = [0u8; 1024];
        {
            let res = iso_msg.get_field(0, &mut buffer);
            assert_eq!(res.unwrap(), 4);
            trace!("mti: {}", str::from_utf8(&buffer[..4]).unwrap());
            assert_eq!(&buffer[..4], "0100".as_bytes());
        }
        {
            let res = iso_msg.get_field(1, &mut buffer);
            assert_eq!(res.unwrap(), 32);
            trace!("bitmap: {}", str::from_utf8(&buffer[..32]).unwrap());
            assert_eq!(&buffer[..32], "F2246481087088360000000000000004".as_bytes());
        }
        {
            trace!("get index 2: card");
            let res = iso_msg.get_field(2, &mut buffer);
            trace!("get index 2: card");
            assert_eq!(res.unwrap(), 16);
            trace!("card: {}", str::from_utf8(&buffer[..16]).unwrap());
            assert_eq!(&buffer[..16], "1234567179299851".as_bytes());
        }
        {
            let res = iso_msg.get_field(3, &mut buffer);
            assert_eq!(res.unwrap(), 6);
            trace!("{}", str::from_utf8(&buffer[..6]).unwrap());
            assert_eq!(&buffer[..6], "003000".as_bytes());
        }

        {
            let res = iso_msg.get_field(4, &mut buffer);
            assert_eq!(res.unwrap(), 12);
            assert_eq!(&buffer[..12], "000000000131".as_bytes());
        }
        {
            let res = iso_msg.get_field(5, &mut buffer);
            assert_eq!(res.is_err(), true);
            assert_eq!(res, Err("Field not set"));
        }
        {
            let _ = iso_msg.set_field(0, "0110".as_bytes());
            let res = iso_msg.get_field(0, &mut buffer);
            assert_eq!(res.unwrap(), 4);
            assert_eq!(&buffer[..4], "0110".as_bytes());


        }

        {
            //remove
            {
                let _ = iso_msg.remove_field(0);
            }
            {
                let res = iso_msg.get_field(0, &mut buffer);
                assert_eq!(res.is_err(), true);
                assert_eq!(res, Err("Field not set"));
            }
            {
                //set
                let _ = iso_msg.set_field(0, "0110".as_bytes());
            } //get
            {
                let res = iso_msg.get_field(0, &mut buffer);
                assert_eq!(res.unwrap(), 4);
                assert_eq!(&buffer[..4], "0110".as_bytes());
            }
        }

    }



    #[test]
    fn iso_to_byte_array_test() {
        let payload = "0100F2246481087088360000000000000004016123456717929985100300000000000013112042128251178162210581284001059006419310712815007743555555555555888Test Merchant         Richmond1    51USA011          N8402001010000000000014510002329467890120100  00054002140000000000012312340001080000000020120040001N 989";
        let handle = AuthSpecs::new();
        let iso_msg = IsoMsg::new(&handle, payload.as_bytes());
        let mut buffer = [0u8; 1024];
        let total_size = iso_msg.to_byte_array(&mut buffer);
        assert_eq!(payload.len(), total_size);
        assert_eq!(str::from_utf8(&buffer[0..total_size]).unwrap(), payload);
    }



    #[test]
    fn iso_auth_req_test() {
        let payload = "0100F2246481087088360000000000000004016123456717929985100300000000000013112042128251178162210581284001059006419310712815007743555555555555888Test Merchant         Richmond1    51USA011          N8402001010000000000014510002329467890120100  00054002140000000000012312340001080000000020120040001N 989";
        let handle = AuthSpecs::new();
        let mut iso_msg = IsoMsg::new(&handle, payload.as_bytes());
        let mut out_buffer = [0u8; 1024];

        // the MTI response to 0100 => 0110
        let mti = String::from("0110");
        {
            let res = iso_msg.set_field(0, &mti.as_bytes()); //set token as pan
            assert_eq!(res, Ok(()));
        }
        //get pan , expiry from api
        let pan = String::from("1234567229741725");
        {
            let res = iso_msg.set_field(2, &pan.as_bytes()); //set token as pan
            assert_eq!(res, Ok(()));
        }

        {
            let expiry = String::from("2202");
            let res = iso_msg.set_field(14, &expiry.as_bytes()); // set token expiry as pan expiry
            assert_eq!(res, Ok(()));
        }


        //set the response code
        {
            let response_code = String::from("00");
            let res = iso_msg.set_field(39, &response_code.as_bytes()); // set response code 00
            assert_eq!(res, Ok(()));
        }

        //verify no changes to bitmap
        {
            let total_size = iso_msg.to_byte_array(&mut out_buffer);

            trace!(
                "iso_msg::to_byte_array:{}",
                str::from_utf8(&out_buffer[..total_size]).unwrap()
            );
            //  assert_eq!(payload.len(), total_size);
            assert_eq!(
                str::from_utf8(&out_buffer[4..36 as usize]).unwrap(),
                "F22464810A7088360000000000000004"
            );
        }



        //remove 126  (it remove last character set:4 in the bitmap )
        {
            let res = iso_msg.remove_field(126); // set token expiry as pan expiry
            assert_eq!(res, Ok(()));
        }

        {
            //verify change in bitmap
            let total_size = iso_msg.to_byte_array(&mut out_buffer);
            assert!(total_size > 0);


            // assert_eq!(out_len as usize, tiso_msg_byte_array.len());
            //  assert_eq!(
            //      str::from_utf8(&out_buffer[4..36 as usize]).unwrap(), /*/*F2246481087088360000000000000004*/*/
            //      "F22464810A7088B60000000000000000"
            //  );

        }

        //set DE44: CVI2 Results Code = M
        {

            let result_code = "          M";
            let res1 = iso_msg.set_field(44, result_code.as_bytes()); // set token expiry as pan expiry
            assert_eq!(res1, Ok(()));
        }


        let tiso_msg_responsebyte_array = "0110F22464810A708836000000000000000001612345672297417250030000000000001311204212825117816220258128400105900641931071281500774300555555555555888Test Merchant         Richmond1    51USA011          M8402001010000000000014510002329467890120100  0005400214000000000001231234000108000000002";
        let total_size = iso_msg.to_byte_array(&mut out_buffer);
        assert_eq!(tiso_msg_responsebyte_array.len(), total_size);
        assert_eq!(
            str::from_utf8(&out_buffer[0..total_size]).unwrap(),
            tiso_msg_responsebyte_array
        );
    }

    extern crate test;
    use self::test::Bencher;

    #[bench]
    fn bench_iso_msg_from_bytearray(b: &mut Bencher) {
        let payload = "0100F2246481087088360000000000000004016123456717929985100300000000000013112042128251178162210581284001059006419310712815007743555555555555888Test Merchant         Richmond1    51USA011          N8402001010000000000014510002329467890120100  00054002140000000000012312340001080000000020120040001N 989";
        let handle = AuthSpecs::new();
        b.iter(|| {
            let _iso_msg = IsoMsg::new(&handle, payload.as_bytes());
        });

    }
    #[bench]
    fn bench_iso_msg_to_bytearray(b: &mut Bencher) {
        let payload = "0100F2246481087088360000000000000004016123456717929985100300000000000013112042128251178162210581284001059006419310712815007743555555555555888Test Merchant         Richmond1    51USA011          N8402001010000000000014510002329467890120100  00054002140000000000012312340001080000000020120040001N 989";
        let handle = AuthSpecs::new();
        let iso_msg = IsoMsg::new(&handle, payload.as_bytes());
        let mut buffer = [0u8; 1024];
        let mut total_size = 0;
        b.iter(|| { total_size = iso_msg.to_byte_array(&mut buffer); });
        assert_eq!(payload.len(), total_size);
        assert_eq!(str::from_utf8(&buffer[0..total_size]).unwrap(), payload);
    }

    #[bench]
    fn bench_iso_msg_to_from_bytearray(b: &mut Bencher) {
        let payload = "0100F2246481087088360000000000000004016123456717929985100300000000000013112042128251178162210581284001059006419310712815007743555555555555888Test Merchant         Richmond1    51USA011          N8402001010000000000014510002329467890120100  00054002140000000000012312340001080000000020120040001N 989";
        let mut buffer = [0u8; 1024];
        let mut total_size = 0;
        let handle = AuthSpecs::new();
        b.iter(|| {
            let iso_msg = IsoMsg::new(&handle, payload.as_bytes());
            total_size = iso_msg.to_byte_array(&mut buffer);
        });
        assert_eq!(payload.len(), total_size);
        assert_eq!(str::from_utf8(&buffer[0..total_size]).unwrap(), payload);
    }





}
