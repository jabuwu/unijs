use serde::{de::DeserializeOwned, Serialize};

use crate::Scope;

#[derive(Clone)]
pub enum Value {
    Undefined,
    Null,
    Bool(bool),
    Number(f64),
    String(String),
    Array(Array),
    Object(Object),
    Function(Function),
}

impl Value {
    #[cfg(not(target_arch = "wasm32"))]
    pub(crate) fn from_v8<'a, 'b>(
        scope: &mut v8::HandleScope<'a>,
        value: v8::Local<'b, v8::Value>,
    ) -> Self {
        if value.is_undefined() {
            Self::Undefined
        } else if value.is_null() {
            Self::Null
        } else if value.is_boolean() {
            Self::Bool(value.boolean_value(scope))
        } else if value.is_number() {
            Self::Number(value.number_value(scope).unwrap())
        } else if value.is_string() {
            // TODO: this impl kinda sucks?
            let string: v8::Local<v8::String> = value.try_into().unwrap();
            let mut buffer = [0; 1024];
            let mut nchars = 0;
            string.write_utf8(
                scope,
                &mut buffer,
                Some(&mut nchars),
                v8::WriteOptions::default(),
            );
            let string = std::str::from_utf8(&buffer).unwrap().to_owned();
            Self::String(string.chars().take(nchars).collect())
        } else if value.is_function() {
            Self::Function(Function::from_v8(scope, value.try_into().unwrap()))
        } else if value.is_array() {
            Self::Array(Array::from_v8(scope, value.try_into().unwrap()))
        } else if value.is_object() {
            Self::Object(Object::from_v8(scope, value.try_into().unwrap()))
        } else {
            todo!("{:?}", value.to_rust_string_lossy(scope))
        }
    }

    #[cfg(not(target_arch = "wasm32"))]
    pub(crate) fn to_v8<'s>(&self, scope: &mut v8::HandleScope<'s>) -> v8::Local<'s, v8::Value> {
        match self {
            Value::Undefined => v8::undefined(scope).into(),
            Value::Null => v8::null(scope).into(),
            Value::Bool(value) => v8::Boolean::new(scope, *value).into(),
            Value::Number(value) => v8::Number::new(scope, *value).into(),
            Value::String(value) => v8::String::new(scope, value.as_str()).unwrap().into(),
            Value::Array(value) => value.to_v8(scope).into(),
            Value::Object(value) => value.to_v8(scope).into(),
            Value::Function(value) => value.to_v8(scope).into(),
        }
    }

    #[cfg(target_arch = "wasm32")]
    pub(crate) fn from_web<'s>(value: wasm_bindgen::JsValue) -> Self {
        if value.is_undefined() {
            Self::Undefined
        } else if value.is_null() {
            Self::Null
        } else if let Some(value) = value.as_bool() {
            Self::Bool(value)
        } else if let Some(value) = value.as_f64() {
            Self::Number(value)
        } else if let Some(value) = value.as_string() {
            Self::String(value)
        } else if value.is_function() {
            Self::Function(Function::from_web(value.into()))
        } else if value.is_array() {
            Self::Array(Array::from_web(value.into()))
        } else if value.is_object() {
            Self::Object(Object::from_web(value.into()))
        } else {
            todo!("{:?}", value)
        }
    }

    #[cfg(target_arch = "wasm32")]
    pub(crate) fn to_web(&self) -> wasm_bindgen::JsValue {
        match self {
            Value::Undefined => wasm_bindgen::JsValue::undefined(),
            Value::Null => wasm_bindgen::JsValue::null(),
            Value::Bool(value) => wasm_bindgen::JsValue::from_bool(*value),
            Value::Number(value) => wasm_bindgen::JsValue::from_f64(*value),
            Value::String(value) => wasm_bindgen::JsValue::from_str(value.as_str()),
            Value::Array(value) => value.to_web().into(),
            Value::Object(value) => value.to_web().into(),
            Value::Function(value) => value.to_web().into(),
        }
    }

    pub fn is_undefined(self) -> bool {
        matches!(self, Self::Undefined)
    }

    pub fn is_null(self) -> bool {
        matches!(self, Self::Null)
    }

    pub fn is_bool(self) -> bool {
        matches!(self, Self::Bool(..))
    }

    pub fn is_number(self) -> bool {
        matches!(self, Self::Number(..))
    }

    pub fn is_string(self) -> bool {
        matches!(self, Self::String(..))
    }

    pub fn is_array(self) -> bool {
        matches!(self, Self::Array(..))
    }

    pub fn is_object(self) -> bool {
        matches!(self, Self::Object(..))
    }

    pub fn is_function(self) -> bool {
        matches!(self, Self::Function(..))
    }

    pub fn into_bool(self) -> Option<bool> {
        if let Value::Bool(bool) = self {
            Some(bool)
        } else {
            None
        }
    }

    pub fn into_number(self) -> Option<f64> {
        if let Value::Number(number) = self {
            Some(number)
        } else {
            None
        }
    }

    pub fn into_string(self) -> Option<String> {
        if let Value::String(string) = self {
            Some(string)
        } else {
            None
        }
    }

    pub fn into_array(self) -> Option<Array> {
        if let Value::Array(array) = self {
            Some(array)
        } else {
            None
        }
    }

    pub fn into_object(self) -> Option<Object> {
        if let Value::Object(object) = self {
            Some(object)
        } else {
            None
        }
    }

    pub fn into_function(self) -> Option<Function> {
        if let Value::Function(function) = self {
            Some(function)
        } else {
            None
        }
    }

    pub fn into_json(self, scope: &mut Scope) -> Option<serde_json::Value> {
        match self {
            Self::Undefined => None,
            Self::Null => Some(serde_json::Value::Null),
            Self::Bool(value) => Some(serde_json::Value::Bool(value)),
            Self::Number(value) => {
                let number = if value as u64 as f64 == value {
                    serde_json::from_str::<serde_json::Number>(&format!("{}", value as u64)).unwrap()
                } else if value as i64 as f64 == value {
                    serde_json::from_str::<serde_json::Number>(&format!("{}", value as i64)).unwrap()
                } else {
                    serde_json::Number::from_f64(value).unwrap()
                };
                Some(serde_json::Value::Number(number))
            }
            Self::String(value) => Some(serde_json::Value::String(value)),
            Self::Array(value) => {
                let mut array = vec![];
                for i in 0..value.length(scope) {
                    let item = value.get(scope, i);
                    if let Some(json) = item.into_json(scope) {
                        array.push(json);
                    } else {
                        array.push(serde_json::Value::Null);
                    }
                }
                Some(serde_json::Value::Array(array))
            }
            Self::Object(value) => {
                let mut map = serde_json::Map::new();
                for key in value.keys(scope) {
                    if let Some(value) = value.get(scope, &key).into_json(scope) {
                        map.insert(key, value);
                    }
                }
                Some(serde_json::Value::Object(map))
            }
            Self::Function(..) => None,
        }
    }

    pub fn from_json(scope: &mut Scope, json: serde_json::Value) -> Self {
        match json {
            serde_json::Value::Null => Self::Null,
            serde_json::Value::Bool(value) => Self::Bool(value),
            serde_json::Value::Number(value) => Self::Number(value.as_f64().unwrap()),
            serde_json::Value::String(value) => Self::String(value),
            serde_json::Value::Array(value) => {
                let array = Array::new(scope);
                for item in value {
                    let item = Value::from_json(scope, item);
                    array.push(scope, item);
                }
                Self::Array(array)
            }
            serde_json::Value::Object(value) => {
                let object = Object::new(scope);
                for (key, value) in value {
                    let item = Value::from_json(scope, value);
                    object.set(scope, &key, item);
                }
                Self::Object(object)
            }
        }
    }

    pub fn serialize<T: Serialize>(scope: &mut Scope, value: &T) -> Option<Self> {
        if let Ok(json) = serde_json::to_value(&value) {
            Some(Self::from_json(scope, json))
        } else {
            None
        }
    }

    pub fn deserialize<T: DeserializeOwned>(self, scope: &mut Scope) -> Option<T> {
        if let Some(json) = self.into_json(scope) {
            serde_json::from_value::<T>(json).ok()
        } else {
            None
        }
    }
}

impl std::fmt::Debug for Value {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Undefined => f.write_str("undefined"),
            Self::Null => f.write_str("null"),
            Self::Bool(value) => value.fmt(f),
            Self::Number(value) => value.fmt(f),
            Self::String(value) => value.fmt(f),
            Self::Array(value) => value.fmt(f),
            Self::Object(value) => value.fmt(f),
            Self::Function(value) => value.fmt(f),
        }
    }
}

#[derive(Clone)]
pub struct Array {
    #[cfg(not(target_arch = "wasm32"))]
    array: v8::Global<v8::Array>,
    #[cfg(target_arch = "wasm32")]
    array: js_sys::Array,
}

impl Array {
    #[cfg(not(target_arch = "wasm32"))]
    pub(crate) fn from_v8<'a, 'b>(
        scope: &mut v8::HandleScope<'a>,
        array: v8::Local<'b, v8::Array>,
    ) -> Self {
        Self {
            array: v8::Global::new(scope, array),
        }
    }

    #[cfg(not(target_arch = "wasm32"))]
    pub(crate) fn to_v8<'s>(&self, scope: &mut v8::HandleScope<'s>) -> v8::Local<'s, v8::Array> {
        v8::Local::new(scope, &self.array)
    }

    #[cfg(target_arch = "wasm32")]
    pub(crate) fn from_web<'s>(array: js_sys::Array) -> Self {
        Self { array }
    }

    #[cfg(target_arch = "wasm32")]
    pub(crate) fn to_web<'s>(&self) -> js_sys::Array {
        self.array.clone()
    }

    #[allow(unused_variables)]
    pub fn new(scope: &mut Scope) -> Self {
        #[cfg(not(target_arch = "wasm32"))]
        {
            let array = scope.enter(|scope| {
                let array = v8::Array::new(scope, 0);
                v8::Global::new(scope, array)
            });
            Self { array }
        }
        #[cfg(target_arch = "wasm32")]
        {
            Self {
                array: js_sys::Array::new(),
            }
        }
    }

    #[allow(unused_variables)]
    pub fn length(&self, scope: &mut Scope) -> u32 {
        #[cfg(not(target_arch = "wasm32"))]
        {
            let array = self.array.clone();
            scope.enter(move |scope| {
                let array = v8::Local::new(scope, array);
                array.length()
            })
        }
        #[cfg(target_arch = "wasm32")]
        {
            self.array.length()
        }
    }

    #[allow(unused_variables)]
    pub fn get(&self, scope: &mut Scope, index: u32) -> Value {
        #[cfg(not(target_arch = "wasm32"))]
        {
            let array = self.array.clone();
            scope.enter(move |scope| {
                let array = v8::Local::new(scope, array);
                let key = v8::Number::new(scope, index as f64);
                if let Some(value) = array.get(scope, key.into()) {
                    Value::from_v8(scope, value)
                } else {
                    Value::Undefined
                }
            })
        }
        #[cfg(target_arch = "wasm32")]
        {
            Value::from_web(self.array.get(index))
        }
    }

    #[allow(unused_variables)]
    pub fn set(&self, scope: &mut Scope, index: u32, value: Value) {
        #[cfg(not(target_arch = "wasm32"))]
        {
            let array = self.array.clone();
            scope.enter(move |scope| {
                let array = v8::Local::new(scope, array);
                let key = v8::Number::new(scope, index as f64);
                let value = value.to_v8(scope);
                array.set(scope, key.into(), value);
            })
        }
        #[cfg(target_arch = "wasm32")]
        {
            self.array.set(index, value.to_web())
        }
    }

    #[allow(unused_variables)]
    pub fn push(&self, scope: &mut Scope, value: Value) {
        #[cfg(not(target_arch = "wasm32"))]
        {
            let array = self.array.clone();
            scope.enter(move |scope| {
                let array = v8::Local::new(scope, array);
                let length = array.length();
                let key = v8::Number::new(scope, length as f64);
                let value = value.to_v8(scope);
                array.set(scope, key.into(), value);
            });
        }
        #[cfg(target_arch = "wasm32")]
        {
            self.array.push(&value.to_web());
        }
    }
}

impl std::fmt::Debug for Array {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&format!("[array]"))
    }
}

impl From<Array> for Value {
    fn from(value: Array) -> Self {
        Value::Array(value)
    }
}

#[derive(Clone)]
pub struct Object {
    #[cfg(not(target_arch = "wasm32"))]
    object: v8::Global<v8::Object>,
    #[cfg(target_arch = "wasm32")]
    object: js_sys::Object,
}

impl Object {
    #[cfg(not(target_arch = "wasm32"))]
    pub(crate) fn from_v8<'a, 'b>(
        scope: &mut v8::HandleScope<'a>,
        object: v8::Local<'b, v8::Object>,
    ) -> Self {
        Self {
            object: v8::Global::new(scope, object),
        }
    }

    #[cfg(not(target_arch = "wasm32"))]
    pub(crate) fn to_v8<'s>(&self, scope: &mut v8::HandleScope<'s>) -> v8::Local<'s, v8::Object> {
        v8::Local::new(scope, &self.object)
    }

    #[cfg(target_arch = "wasm32")]
    pub(crate) fn from_web<'s>(object: js_sys::Object) -> Self {
        Self { object }
    }

    #[cfg(target_arch = "wasm32")]
    pub(crate) fn to_web<'s>(&self) -> js_sys::Object {
        self.object.clone()
    }

    #[allow(unused_variables)]
    pub fn new(scope: &mut Scope) -> Self {
        #[cfg(not(target_arch = "wasm32"))]
        {
            let object = scope.enter(|scope| {
                let object = v8::Object::new(scope);
                v8::Global::new(scope, object)
            });
            Self { object }
        }
        #[cfg(target_arch = "wasm32")]
        {
            Self {
                object: js_sys::Object::new(),
            }
        }
    }

    #[allow(unused_variables)]
    pub fn get(&self, scope: &mut Scope, name: &str) -> Value {
        #[cfg(not(target_arch = "wasm32"))]
        {
            let object = self.object.clone();
            scope.enter(move |scope| {
                let object = v8::Local::new(scope, object);
                let name = v8::String::new(scope, name).unwrap();
                let value = object.get(scope, name.into());
                if let Some(value) = value {
                    Value::from_v8(scope, value)
                } else {
                    Value::Undefined
                }
            })
        }
        #[cfg(target_arch = "wasm32")]
        {
            Value::from_web(
                js_sys::Reflect::get(&self.object, &wasm_bindgen::JsValue::from(name)).unwrap(),
            )
        }
    }

    #[allow(unused_variables)]
    pub fn set(&self, scope: &mut Scope, name: &str, value: Value) {
        #[cfg(not(target_arch = "wasm32"))]
        {
            let object = self.object.clone();
            scope.enter(move |scope| {
                let object = v8::Local::new(scope, object);
                let name = v8::String::new(scope, name).unwrap();
                let value = value.to_v8(scope);
                object.set(scope, name.into(), value).unwrap();
            })
        }
        #[cfg(target_arch = "wasm32")]
        {
            js_sys::Reflect::set(
                &self.object,
                &wasm_bindgen::JsValue::from(name),
                &value.to_web(),
            )
            .unwrap();
        }
    }

    #[allow(unused_variables)]
    pub fn keys(&self, scope: &mut Scope) -> Vec<String> {
        #[cfg(not(target_arch = "wasm32"))]
        {
            let object = self.object.clone();
            scope.enter(move |scope| {
                let object = v8::Local::new(scope, object);
                let names = object.get_own_property_names(scope, v8::GetPropertyNamesArgs::default()).unwrap();
                let mut keys = vec![];
                for i in 0..names.length() {
                    let i_key = v8::Number::new(scope, i as f64);
                    if let Some(name) = names.get(scope, i_key.into()) {
                        let name = Value::from_v8(scope, name);
                        if let Some(name) = name.into_string() {
                            keys.push(name);
                        }
                    }
                }
                keys
            })
        }
        #[cfg(target_arch = "wasm32")]
        {
            let mut keys = vec![];
            let object_keys = js_sys::Reflect::own_keys(&self.object.clone().into()).unwrap();
            for item in object_keys {
                if let Some(name) = item.as_string() {
                    keys.push(name);
                }
            }
            keys
        }
    }
}

impl std::fmt::Debug for Object {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("[object]")
    }
}

impl From<Object> for Value {
    fn from(value: Object) -> Self {
        Value::Object(value)
    }
}

#[derive(Clone)]
pub struct Function {
    #[cfg(not(target_arch = "wasm32"))]
    function: v8::Global<v8::Function>,
    #[cfg(target_arch = "wasm32")]
    function: js_sys::Function,
}

impl Function {
    #[cfg(not(target_arch = "wasm32"))]
    pub(crate) fn from_v8<'a, 'b>(
        scope: &mut v8::HandleScope<'a>,
        function: v8::Local<'b, v8::Function>,
    ) -> Self {
        Self {
            function: v8::Global::new(scope, function),
        }
    }

    #[cfg(not(target_arch = "wasm32"))]
    pub(crate) fn to_v8<'s>(&self, scope: &mut v8::HandleScope<'s>) -> v8::Local<'s, v8::Function> {
        v8::Local::new(scope, &self.function)
    }

    #[cfg(target_arch = "wasm32")]
    pub(crate) fn from_web<'s>(function: js_sys::Function) -> Self {
        Self { function }
    }

    #[cfg(target_arch = "wasm32")]
    pub(crate) fn to_web<'s>(&self) -> js_sys::Function {
        self.function.clone()
    }

    #[allow(unused_variables)]
    pub fn new(scope: &mut Scope, f: fn(&mut Scope, Args) -> Value) -> Self {
        #[cfg(not(target_arch = "wasm32"))]
        {
            let function = scope.enter(|scope| {
                let f_ptr = v8::Number::new(scope, f as usize as f64);
                let function = v8::Function::builder(
                    |v8_scope: &mut v8::HandleScope<'_>,
                     v8_args: v8::FunctionCallbackArguments<'_>,
                     mut v8_ret: v8::ReturnValue<'_>| {
                        let f: fn(&mut Scope, Args) -> Value = unsafe {
                            std::mem::transmute(
                                v8_args.data().number_value(v8_scope).unwrap() as usize
                            )
                        };
                        let mut args = Args { args: vec![] };
                        for i in 0..v8_args.length() {
                            args.args.push(Value::from_v8(v8_scope, v8_args.get(i)));
                        }
                        let value = {
                            let mut scope = Scope::scope(v8_scope);
                            f(&mut scope, args)
                        };
                        v8_ret.set(value.to_v8(v8_scope));
                    },
                )
                .data(f_ptr.into())
                .build(scope)
                .unwrap();
                v8::Global::new(scope, function)
            });
            Self { function }
        }
        #[cfg(target_arch = "wasm32")]
        {
            use js_sys::{Array, Reflect};
            use wasm_bindgen::{closure::Closure, JsCast, JsValue};
            use web_sys::window;
            let f_ptr = f as usize;
            let handle_str = format!("__fn_{}", f_ptr);
            let handle = JsValue::from(&handle_str);
            if Reflect::get(&window().into(), &JsValue::from(&handle))
                .unwrap()
                .is_undefined()
            {
                let closure =
                    Closure::<dyn Fn(JsValue) -> JsValue>::new(move |js_args: JsValue| {
                        let mut scope = Scope::new();
                        let js_args_array: Array = js_args.into();
                        let mut args = Args { args: vec![] };
                        for i in 0..js_args_array.length() {
                            args.args.push(Value::from_web(js_args_array.get(i)));
                        }
                        let ret = f(&mut scope, args);
                        ret.to_web()
                    });
                Reflect::set(&window().into(), &handle, closure.as_ref().unchecked_ref()).unwrap();
                closure.forget();
            }
            Function::from_web(js_sys::eval(&format!("function args_wrapper() {{ return window.{}.apply(null, [Array.from(arguments)]); }}; args_wrapper", &handle_str)).unwrap().try_into().unwrap())
        }
    }

    #[allow(unused_variables)]
    pub fn call(&self, scope: &mut Scope, args: &[Value]) -> Value {
        #[cfg(not(target_arch = "wasm32"))]
        {
            let function = self.function.clone();
            scope.enter(move |scope| {
                let function = v8::Local::new(scope, function);
                let recv = v8::null(scope);
                let args = args
                    .iter()
                    .map(|value| value.to_v8(scope))
                    .collect::<Vec<_>>();
                let ret = function.call(scope, recv.into(), &args).unwrap();
                Value::from_v8(scope, ret)
            })
        }
        #[cfg(target_arch = "wasm32")]
        {
            let array = js_sys::Array::new();
            for arg in args {
                array.push(&arg.to_web());
            }
            let ret = self
                .function
                .apply(&wasm_bindgen::JsValue::null(), &array)
                .unwrap();
            Value::from_web(ret)
        }
    }
}

impl std::fmt::Debug for Function {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("[function]")
    }
}

impl From<Function> for Value {
    fn from(value: Function) -> Self {
        Value::Function(value)
    }
}

pub struct Args {
    pub(crate) args: Vec<Value>,
}

impl Args {
    pub fn get(&self, index: usize) -> Value {
        self.args
            .get(index)
            .cloned()
            .unwrap_or_else(|| Value::Undefined)
    }

    pub fn length(&self) -> u32 {
        self.args.len() as u32
    }
}
