mod backend;

use std::time::Duration;

use backend::{MultimediaBackend, OutputItem};
use neon::{prelude::*, types::buffer::TypedArray};
use uiua::{encode::SmartOutput, format::FormatConfig, *};

fn eval_internal(
    mut uiua: Uiua,
    cx: &mut FunctionContext,
) -> NeonResult<(Uiua, Result<Compiler, UiuaError>)> {
    let code = cx.argument::<JsString>(0)?.value(cx);
    let mut experimental = false;

    if let Some(o) = cx.argument_opt(1) {
        let obj = o
            .downcast_or_throw::<JsObject, FunctionContext>(cx)
            .unwrap();
        if let Some(timeout_opt) = obj.get_opt::<JsNumber, _, _>(cx, "timeout").unwrap() {
            uiua = uiua.with_execution_limit(Duration::from_millis(timeout_opt.value(cx) as u64));
        }
        if let Some(experimental_opt) = obj.get_opt::<JsBoolean, _, _>(cx, "experimental").unwrap()
        {
            experimental = experimental_opt.value(cx);
        }
        if let Some(inputs) = obj.get_opt::<JsArray, _, _>(cx, "inputs").unwrap() {
            for input in inputs.to_vec(cx)?.iter().rev() {
                if let Ok(str) = input.downcast::<JsString, _>(cx) {
                    uiua.push(str.value(cx));
                } else if let Ok(num) = input.downcast::<JsNumber, _>(cx) {
                    uiua.push(num.value(cx));
                } else if let Ok(buf) = input.downcast::<JsBuffer, _>(cx) {
                    uiua.push(buf.as_slice(cx).to_vec());
                } else {
                    return cx.throw_type_error("Input must be a string, number, or buffer");
                }
            }
        }
    }

    let res = uiua.compile_run(|comp| {
        if experimental {
            comp.experimental(true);
        }
        comp.load_str(&code)
    });

    Ok((uiua, res))
}

fn eval(mut cx: FunctionContext) -> JsResult<JsObject> {
    let obj = cx.empty_object();
    let res = eval_internal(Uiua::with_safe_sys(), &mut cx).unwrap();
    let mut uiua = res.0;

    let sys: SafeSys = uiua.take_backend().unwrap();
    match res.1 {
        Ok(_) => {
            for value in uiua.take_stack() {
                sys.show(value).unwrap();
            }
        }
        Err(err) => {
            let err_s = cx.string(err.to_string());
            obj.set(&mut cx, "error", err_s)?;
        }
    }

    let stdout = sys.take_stdout();
    let stderr = sys.take_stderr();

    let stdout_b = JsBuffer::from_slice(&mut cx, &stdout)?;
    let stderr_b = JsBuffer::from_slice(&mut cx, &stderr)?;

    obj.set(&mut cx, "stdout", stdout_b)?;
    obj.set(&mut cx, "stderr", stderr_b)?;
    Ok(obj)
}

fn eval_mm(mut cx: FunctionContext) -> JsResult<JsObject> {
    let obj = cx.empty_object();
    let res = eval_internal(Uiua::with_backend(MultimediaBackend::new()), &mut cx).unwrap();
    let mut uiua = res.0;

    let sys: MultimediaBackend = uiua.take_backend().unwrap();
    match res.1 {
        Ok(_) => {
            for value in uiua.take_stack() {
                match SmartOutput::from_value(value, &sys) {
                    SmartOutput::Normal(value) => {
                        sys.show(value).unwrap();
                    }
                    SmartOutput::Png(bytes, label) => {
                        sys.show_png(bytes, label.as_deref()).unwrap();
                    }
                    SmartOutput::Gif(bytes, label) => {
                        sys.show_gif(bytes, label.as_deref()).unwrap();
                    }
                    SmartOutput::Wav(bytes, label) => {
                        sys.play_audio(bytes, label.as_deref()).unwrap();
                    }
                };
            }
        }
        Err(err) => {
            let err_s = cx.string(err.to_string());
            obj.set(&mut cx, "error", err_s)?;
        }
    }

    let outputs = sys.take_outputs();
    let outputs_arr = cx.empty_array();
    for (i, output) in outputs.into_iter().enumerate() {
        let output_obj = cx.empty_object();
        match output {
            OutputItem::StdOut(s) => {
                let t = cx.string("stdout");
                let b = cx.string(s);
                output_obj.set(&mut cx, "type", t)?;
                output_obj.set(&mut cx, "content", b)?;
            }
            OutputItem::StdErr(s) => {
                let t = cx.string("stderr");
                let b = cx.string(s);
                output_obj.set(&mut cx, "type", t)?;
                output_obj.set(&mut cx, "content", b)?;
            }
            OutputItem::Trace(s) => {
                let t = cx.string("trace");
                let b = cx.string(s);
                output_obj.set(&mut cx, "type", t)?;
                output_obj.set(&mut cx, "content", b)?;
            }
            OutputItem::Image { data, mime, label } => {
                let t = cx.string("image");
                let b = JsBuffer::from_slice(&mut cx, &data)?;
                let m = cx.string(mime);
                output_obj.set(&mut cx, "type", t)?;
                output_obj.set(&mut cx, "data", b)?;
                output_obj.set(&mut cx, "mime", m)?;
                if let Some(label) = label {
                    let l = cx.string(label);
                    output_obj.set(&mut cx, "label", l)?;
                }
            }
            OutputItem::Audio { data, mime, label } => {
                let t = cx.string("audio");
                let b = JsBuffer::from_slice(&mut cx, &data)?;
                let m = cx.string(mime);
                output_obj.set(&mut cx, "type", t)?;
                output_obj.set(&mut cx, "data", b)?;
                output_obj.set(&mut cx, "mime", m)?;
                if let Some(label) = label {
                    let l = cx.string(label);
                    output_obj.set(&mut cx, "label", l)?;
                }
            }
        }
        outputs_arr.set(&mut cx, i as u32, output_obj)?;
    }
    obj.set(&mut cx, "outputs", outputs_arr)?;
    Ok(obj)
}

fn format(mut cx: FunctionContext) -> JsResult<JsString> {
    let code = cx.argument::<JsString>(0)?.value(&mut cx);
    let config = FormatConfig::default();
    let result = format::format_str(&code, &config).unwrap();
    Ok(cx.string(result.output))
}

#[neon::main]
fn main(mut cx: ModuleContext) -> NeonResult<()> {
    cx.export_function("eval", eval)?;
    cx.export_function("eval_mm", eval_mm)?;
    cx.export_function("format", format)?;
    Ok(())
}
