use genco::{prelude::js::Tokens, quote};

use crate::{
    code_gen::generateable::VariablePath,
    type_info::{bool_to_js_bool, NumberMeta},
};

use super::{des, JsTypeGenerateable};

impl JsTypeGenerateable for NumberMeta {
    fn gen_ser_accessor(&self, variable_path: VariablePath) -> Tokens {
        let byte_amount_str = self.as_byte_js_string();
        match self {
            NumberMeta::FloatingPoint { .. } => {
                quote!(s.serialize_number_float($byte_amount_str, $variable_path))
            }
            NumberMeta::Integer { signed, .. } => {
                let signed = bool_to_js_bool(*signed);
                quote!(s.serialize_number($byte_amount_str, $signed, $variable_path))
            }
        }
    }

    fn gen_des_accessor(&self, field_accessor: des::FieldAccessor) -> Tokens {
        let byte_amount_str = self.as_byte_js_string();
        match self {
            NumberMeta::FloatingPoint { .. } => {
                quote!($(field_accessor)d.deserialize_number_float($byte_amount_str))
            }
            NumberMeta::Integer { signed, .. } => {
                let signed = bool_to_js_bool(*signed);
                quote!($(field_accessor)d.deserialize_number($byte_amount_str, $signed))
            }
        }
    }

    fn gen_ty_check(&self, variable_path: VariablePath) -> Tokens {
        let byte_amount_str = self.as_byte_js_string();
        match self {
            NumberMeta::FloatingPoint { .. } => {
                quote!(typeof $(variable_path.to_owned()) === "number" && !Number.isInteger($(variable_path.to_owned())) && Number.isFinite($(variable_path.to_owned())))
            }
            NumberMeta::Integer { signed, .. } => {
                let signed = bool_to_js_bool(*signed);
                quote!((typeof $(variable_path.to_owned()) === "number" || typeof $(variable_path.to_owned()) === "bigint") && Number.isInteger($(variable_path.to_owned())) && check_bounds($byte_amount_str, $signed, $variable_path))
            }
        }
    }

    fn gen_ts_type(&self) -> Tokens {
        match self {
            NumberMeta::FloatingPoint { bytes } => {
                let bits = match bytes {
                    4 => "32",
                    8 => "64",
                    _ => unreachable!(),
                };
                quote!(f$bits)
            }
            NumberMeta::Integer { bytes, signed } => {
                let prefix = if *signed { "i" } else { "u" };
                let bits = match bytes {
                    1 => "8",
                    2 => "16",
                    4 => "32",
                    8 => "64",
                    16 => "128",
                    _ => unreachable!(),
                };
                quote!($prefix$bits)
            }
        }
    }
}
