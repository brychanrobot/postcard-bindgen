use genco::{prelude::js::Tokens, quote};

use crate::{
    code_gen::{utils::semicolon_chain, JS_OBJECT_VARIABLE},
    type_info::{
        bool_to_js_bool, ArrayMeta, JsType, NumberMeta, ObjectMeta, OptionalMeta, RangeMeta,
        StringMeta,
    },
    utils::StrExt,
};

use super::{
    des, ser,
    ty_check::{self, AvailableCheck},
    AccessorGenerateable,
};

impl AccessorGenerateable for JsType {
    fn gen_ser_accessor(&self, variable_path: ser::VariablePath) -> Tokens {
        match self {
            Self::Number(number_meta) => number_meta.gen_ser_accessor(variable_path),
            Self::Array(array_meta) => array_meta.gen_ser_accessor(variable_path),
            Self::Object(object_meta) => object_meta.gen_ser_accessor(variable_path),
            Self::Optional(optional_meta) => optional_meta.gen_ser_accessor(variable_path),
            Self::String(string_meta) => string_meta.gen_ser_accessor(variable_path),
            Self::Range(range_meta) => range_meta.gen_ser_accessor(variable_path),
        }
    }

    fn gen_des_accessor(&self, field_accessor: des::FieldAccessor) -> Tokens {
        match self {
            Self::Number(number_meta) => number_meta.gen_des_accessor(field_accessor),
            Self::Array(array_meta) => array_meta.gen_des_accessor(field_accessor),
            Self::Object(object_meta) => object_meta.gen_des_accessor(field_accessor),
            Self::Optional(optional_meta) => optional_meta.gen_des_accessor(field_accessor),
            Self::String(string_meta) => string_meta.gen_des_accessor(field_accessor),
            Self::Range(range_meta) => range_meta.gen_des_accessor(field_accessor),
        }
    }

    fn gen_ty_check(&self, variable_path: ser::VariablePath) -> Tokens {
        match self {
            Self::Number(number_meta) => number_meta.gen_ty_check(variable_path),
            Self::Array(array_meta) => array_meta.gen_ty_check(variable_path),
            Self::Object(object_meta) => object_meta.gen_ty_check(variable_path),
            Self::Optional(optional_meta) => optional_meta.gen_ty_check(variable_path),
            Self::String(string_meta) => string_meta.gen_ty_check(variable_path),
            Self::Range(range_meta) => range_meta.gen_ty_check(variable_path),
        }
    }
}

impl AccessorGenerateable for RangeMeta {
    fn gen_ser_accessor(&self, variable_path: ser::VariablePath) -> Tokens {
        let start_path = variable_path
            .to_owned()
            .modify_push(ser::VariableAccess::Field("start".into()));
        let stop_path = variable_path.modify_push(ser::VariableAccess::Field("stop".into()));

        let start_accessor = self.bounds_type.gen_ser_accessor(start_path);
        let stop_accessor = self.bounds_type.gen_ser_accessor(stop_path);

        semicolon_chain([start_accessor, stop_accessor])
    }

    fn gen_des_accessor(&self, field_accessor: des::FieldAccessor) -> Tokens {
        let field_des = self.bounds_type.gen_des_accessor(des::FieldAccessor::None);
        quote!($field_accessor{ start: $(field_des.to_owned()), stop: $field_des })
    }

    fn gen_ty_check(&self, variable_path: ser::VariablePath) -> Tokens {
        quote!(typeof $(variable_path.to_owned()) === "object" && "start" in $(variable_path.to_owned()) && "stop" in $variable_path)
    }
}

impl AccessorGenerateable for OptionalMeta {
    fn gen_ser_accessor(&self, variable_path: ser::VariablePath) -> Tokens {
        let type_accessor = self.inner.gen_ser_accessor(variable_path.to_owned());
        quote!(if ($variable_path !== undefined) { s.serialize_number(U32_BYTES, false, 1); $type_accessor } else { s.serialize_number(U32_BYTES, false, 0) })
    }

    fn gen_des_accessor(&self, field_accessor: des::FieldAccessor) -> Tokens {
        let inner_accessor = self.inner.gen_des_accessor(des::FieldAccessor::None);
        quote!($(field_accessor)(d.deserialize_number(U32_BYTES, false) === 0) ? undefined : $inner_accessor)
    }

    fn gen_ty_check(&self, variable_path: ser::VariablePath) -> Tokens {
        let available_check =
            ty_check::AvailableCheck::from_variable_path(variable_path.to_owned());
        let inner_type_check = self.inner.gen_ty_check(variable_path.to_owned());
        match &available_check {
            AvailableCheck::Object(_, _) => {
                quote!((($(available_check.to_owned()) && ($(variable_path.to_owned()) !== undefined && $inner_type_check) || $variable_path === undefined) || !($available_check)))
            }
            AvailableCheck::None => {
                quote!(($(variable_path.to_owned()) !== undefined && $inner_type_check) || $variable_path === undefined)
            }
        }
    }
}

impl AccessorGenerateable for StringMeta {
    fn gen_ser_accessor(&self, variable_path: ser::VariablePath) -> Tokens {
        quote!(s.serialize_string($variable_path))
    }

    fn gen_des_accessor(&self, field_accessor: des::FieldAccessor) -> Tokens {
        quote!($(field_accessor)d.deserialize_string())
    }

    fn gen_ty_check(&self, variable_path: ser::VariablePath) -> Tokens {
        quote!(typeof $variable_path === "string")
    }
}

impl AccessorGenerateable for ObjectMeta {
    fn gen_ser_accessor(&self, variable_path: ser::VariablePath) -> Tokens {
        let obj_ident = self.name.to_obj_identifier();
        quote!(serialize_$obj_ident(s, $variable_path))
    }

    fn gen_des_accessor(&self, field_accessor: des::FieldAccessor) -> Tokens {
        let obj_ident = self.name.to_obj_identifier();
        quote!($(field_accessor)deserialize_$obj_ident(d))
    }

    fn gen_ty_check(&self, variable_path: ser::VariablePath) -> Tokens {
        let obj_ident = self.name.to_obj_identifier();
        quote!(is_$obj_ident($variable_path))
    }
}

impl AccessorGenerateable for ArrayMeta {
    fn gen_ser_accessor(&self, variable_path: ser::VariablePath) -> Tokens {
        let inner_type_accessor = self
            .items_type
            .gen_ser_accessor(ser::VariablePath::default());
        quote!(s.serialize_array((s, $JS_OBJECT_VARIABLE) => $inner_type_accessor, $variable_path))
    }

    fn gen_des_accessor(&self, field_accessor: des::FieldAccessor) -> Tokens {
        let inner_type_accessor = self.items_type.gen_des_accessor(des::FieldAccessor::Array);
        quote!($(field_accessor)d.deserialize_array(() => $inner_type_accessor))
    }

    fn gen_ty_check(&self, variable_path: ser::VariablePath) -> Tokens {
        quote!(Array.isArray($variable_path))
    }
}

impl AccessorGenerateable for NumberMeta {
    fn gen_ser_accessor(&self, variable_path: ser::VariablePath) -> Tokens {
        let byte_amount_str = self.as_byte_js_string();
        let signed = bool_to_js_bool(self.signed);
        quote!(s.serialize_number($byte_amount_str, $signed, $variable_path))
    }

    fn gen_des_accessor(&self, field_accessor: des::FieldAccessor) -> Tokens {
        let byte_amount_str = self.as_byte_js_string();
        let signed = bool_to_js_bool(self.signed);
        quote!($(field_accessor)d.deserialize_number($byte_amount_str, $signed))
    }

    fn gen_ty_check(&self, variable_path: ser::VariablePath) -> Tokens {
        quote!(typeof $variable_path === "number")
    }
}