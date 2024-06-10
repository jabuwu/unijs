#[cfg(not(target_arch = "wasm32"))]
mod native {
    use crate::Object;

    pub fn init() {
        let platform = v8::new_default_platform(0, false).make_shared();
        v8::V8::initialize_platform(platform);
        v8::V8::initialize();
    }

    pub struct Module {
        _private: (),
    }

    impl Module {
        pub fn load<'a, 'b, 'c>(js: &'c str) -> (Scope<'a, 'b>, Object) {
            let mut isolate = v8::Isolate::new(v8::CreateParams::default());
            let exports = {
                let handle_scope = &mut v8::HandleScope::new(&mut isolate);
                let context = v8::Context::new(handle_scope);
                let scope: &mut v8::ContextScope<v8::HandleScope<_>> =
                    &mut v8::ContextScope::new(handle_scope, context);
                let exports = v8::Object::new(scope);
                let exports_key = v8::String::new(scope, "exports").unwrap();
                context
                    .global(scope)
                    .set(scope, exports_key.into(), exports.into());
                let code = v8::String::new(scope, &js).unwrap();
                let script = v8::Script::compile(scope, code, None).unwrap();
                script.run(scope).unwrap();
                Object::from_v8(scope, exports)
            };
            unsafe {
                isolate.exit();
            }
            (Scope(InnerScope::Isolate(isolate)), exports)
        }
    }

    pub struct Scope<'a, 'b>(pub(crate) InnerScope<'a, 'b>);

    pub(crate) enum InnerScope<'a, 'b> {
        Isolate(v8::OwnedIsolate),
        Scope(&'a mut v8::HandleScope<'b>),
    }

    impl<'a, 'b> Scope<'a, 'b> {
        pub(crate) fn scope(scope: &'a mut v8::HandleScope<'b>) -> Self {
            Self(InnerScope::Scope(scope))
        }

        pub(crate) fn enter<F, R>(&mut self, f: F) -> R
        where
            F: FnOnce(&mut v8::HandleScope<v8::Context>) -> R,
        {
            match &mut self.0 {
                InnerScope::Isolate(isolate) => {
                    unsafe {
                        isolate.enter();
                    }
                    let result = {
                        let handle_scope = &mut v8::HandleScope::new(isolate);
                        let context = v8::Context::new(handle_scope);
                        let scope: &mut v8::ContextScope<v8::HandleScope<_>> =
                            &mut v8::ContextScope::new(handle_scope, context);
                        f(scope)
                    };
                    unsafe {
                        isolate.exit();
                    }
                    result
                }
                InnerScope::Scope(scope) => f(*scope),
            }
        }
    }

    impl<'a, 'b> Drop for Scope<'a, 'b> {
        fn drop(&mut self) {
            if let InnerScope::Isolate(isolate) = &mut self.0 {
                unsafe {
                    isolate.enter();
                }
            }
        }
    }
}
#[cfg(not(target_arch = "wasm32"))]
pub use native::*;

#[cfg(target_arch = "wasm32")]
mod wasm {
    use std::marker::PhantomData;

    use js_sys::{eval, Reflect};
    use wasm_bindgen::JsValue;
    use web_sys::{console, window};

    use crate::Object;

    pub fn init() {}

    pub struct Module {
        _private: (),
    }

    impl Module {
        pub fn load<'a, 'b, 'c>(js: &'c str) -> (Scope<'a, 'b>, Object) {
            let exports = js_sys::Object::new();
            Reflect::set(
                &window().unwrap().into(),
                &JsValue::from("exports"),
                &exports,
            )
            .unwrap();
            if let Err(err) = eval(js) {
                console::error_1(&err);
                panic!();
            }
            Reflect::delete_property(&window().unwrap().into(), &JsValue::from("exports")).unwrap();
            (Scope::new(), Object::from_web(exports))
        }
    }

    pub struct Scope<'a, 'b> {
        _a: PhantomData<&'a ()>,
        _b: PhantomData<&'b ()>,
    }

    impl<'a, 'b> Scope<'a, 'b> {
        pub(crate) fn new() -> Self {
            Self {
                _a: PhantomData,
                _b: PhantomData,
            }
        }
    }
}
#[cfg(target_arch = "wasm32")]
pub use wasm::*;
