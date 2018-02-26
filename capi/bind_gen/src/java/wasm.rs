use std::io::{BufWriter, Result, Write};
use {Class, Function, Type, TypeKind};
use heck::MixedCase;
use std::collections::BTreeMap;
use std::path::Path;
use std::fs::File;
use super::write_class_comments;

fn get_hl_type(ty: &Type) -> String {
    if ty.is_custom {
        match ty.kind {
            TypeKind::Ref => format!("{}Ref", ty.name),
            TypeKind::RefMut => format!("{}RefMut", ty.name),
            TypeKind::Value => ty.name.clone(),
        }
    } else if ty.name == "bool" {
        "boolean".to_string()
    } else if ty.name == "usize" {
        "int".to_string()
    } else {
        get_ll_type(ty).to_string()
    }
}

fn get_ll_type(ty: &Type) -> &str {
    match (ty.kind, ty.name.as_str()) {
        (TypeKind::Ref, "c_char") => "String",
        (TypeKind::Ref, _) | (TypeKind::RefMut, _) => "int",
        (_, t) if !ty.is_custom => match t {
            "i8" => "byte",
            "i16" => "short",
            "i32" => "int",
            "i64" => "long",
            "u8" => "byte",
            "u16" => "short",
            "u32" => "int",
            "u64" => "long",
            "usize" => "int",
            "f32" => "float",
            "f64" => "double",
            "bool" => "boolean",
            "()" => "void",
            "c_char" => "byte",
            "Json" => "String",
            x => x,
        },
        _ => "int",
    }
}

fn write_fn<W: Write>(mut writer: W, function: &Function, class_name: &str) -> Result<()> {
    let is_static = function.is_static();
    let has_return_type = function.has_return_type();
    let return_type = get_hl_type(&function.output);
    let is_constructor = function.method == "new" && !function.output.is_nullable;
    let mut method = function.method.to_mixed_case();
    if method == "clone" {
        method = "copy".into();
    } else if method == "close" {
        method = "finish".into();
    } else if method == "new" {
        method = "create".into();
    } else if method == "default" {
        method = "createDefault".into();
    }

    if !function.comments.is_empty() {
        write!(
            writer,
            r#"
    /**"#
        )?;

        for comment in &function.comments {
            write!(
                writer,
                r#"
     * {}"#,
                comment
                    .replace("<NULL>", "null")
                    .replace("<TRUE>", "true")
                    .replace("<FALSE>", "false")
            )?;
        }

        write!(
            writer,
            r#"
     */"#
        )?;
    }

    if is_constructor {
        write!(
            writer,
            r#"
    public {}("#,
            class_name
        )?;
    } else {
        write!(
            writer,
            r#"
    public{} {} {}("#,
            if is_static { " static" } else { "" },
            return_type,
            method
        )?;
    }

    for (i, &(ref name, ref typ)) in function
        .inputs
        .iter()
        .skip(if is_static { 0 } else { 1 })
        .enumerate()
    {
        if i != 0 {
            write!(writer, ", ")?;
        }
        write!(writer, "{} {}", get_hl_type(typ), name.to_mixed_case())?;
    }

    if is_constructor {
        write!(
            writer,
            r#") {{
        super(0);
        "#
        )?;
    } else {
        write!(
            writer,
            r#") {{
        "#
        )?;
    }

    for &(ref name, ref typ) in function.inputs.iter() {
        if typ.is_custom {
            write!(
                writer,
                r#"if ({name}.ptr == 0) {{
            throw new RuntimeException();
        }}
        "#,
                name = name.to_mixed_case()
            )?;
        }
    }

    for &(ref name, ref typ) in function.inputs.iter() {
        let hl_type = get_hl_type(typ);
        if hl_type == "String" || typ.name == "Json" {
            write!(
                writer,
                r#"LiveSplitCoreNative.AllocatedBuf {0}_Allocated = LiveSplitCoreNative.allocString({0});
        "#,
                name.to_mixed_case()
            )?;
        }
    }

    if has_return_type {
        if is_constructor {
            write!(writer, "this.ptr = ")?;
        } else if return_type == "String" || function.output.name == "Json" {
            write!(
                writer,
                "{} result = LiveSplitCoreNative.readString(",
                return_type
            )?;
        } else if function.output.is_custom {
            write!(
                writer,
                r#"{ret_type} result = new {ret_type}("#,
                ret_type = return_type
            )?;
        } else {
            write!(writer, "{} result = ", return_type)?;
            if function.output.name == "u8" {
                write!(writer, "({})", return_type)?;
            }
        }
    }

    write!(
        writer,
        r#"LiveSplitCoreNative.INSTANCE.{}("#,
        &function.name
    )?;

    for (i, &(ref name, ref typ)) in function.inputs.iter().enumerate() {
        if i != 0 {
            write!(writer, ", ")?;
        }
        let hl_ty_name = get_hl_type(typ);
        write!(
            writer,
            "{}",
            if name == "this" {
                "this.ptr".to_string()
            } else if hl_ty_name == "boolean" {
                format!("(byte)({} ? 1 : 0)", name.to_mixed_case())
            } else if hl_ty_name == "String" || typ.name == "Json" {
                format!("{}_Allocated.ptr", name.to_mixed_case())
            } else if typ.is_custom {
                format!("{}.ptr", name.to_mixed_case())
            } else {
                name.to_mixed_case()
            }
        )?;
    }

    write!(
        writer,
        "){}",
        if return_type == "boolean" {
            " != 0"
        } else {
            ""
        }
    )?;

    if return_type == "String" || function.output.name == "Json" {
        write!(writer, r#")"#)?;
    } else if !is_constructor && has_return_type && function.output.is_custom {
        write!(writer, r#")"#)?;
    }

    write!(writer, r#";"#)?;

    for &(ref name, ref typ) in function.inputs.iter() {
        let hl_type = get_hl_type(typ);
        if hl_type == "String" || typ.name == "Json" {
            write!(
                writer,
                r#"
        {}_Allocated.dealloc();"#,
                name.to_mixed_case()
            )?;
        }
    }

    for &(ref name, ref typ) in function.inputs.iter() {
        if typ.is_custom && typ.kind == TypeKind::Value {
            write!(
                writer,
                r#"
        {}.ptr = 0;"#,
                name.to_mixed_case()
            )?;
        }
    }

    if has_return_type && !is_constructor {
        if function.output.is_nullable && function.output.is_custom {
            write!(
                writer,
                r#"
        if (result.ptr == 0) {{
            return null;
        }}"#
            )?;
        }
        write!(
            writer,
            r#"
        return result;"#
        )?;
    }

    write!(
        writer,
        r#"
    }}"#
    )?;

    Ok(())
}

fn write_class_ref<P: AsRef<Path>>(path: P, class_name: &str, class: &Class) -> Result<()> {
    let mut writer = BufWriter::new(File::create(path)?);
    let class_name_ref = format!("{}Ref", class_name);

    write!(
        writer,
        r#"package org.livesplit;
"#
    )?;

    write_class_comments(&mut writer, &class.comments)?;

    write!(
        writer,
        r#"
public class {class} {{
    int ptr;"#,
        class = class_name_ref
    )?;

    for function in &class.shared_fns {
        write_fn(&mut writer, function, &class_name_ref)?;
    }

    if class_name == "SharedTimer" {
        write!(
            writer,
            "{}",
            r#"
    public void readWith(java.util.function.Consumer<TimerRef> action) {
        try (TimerReadLock timerLock = read()) {
            action.accept(timerLock.timer());
        }
    }
    public void writeWith(java.util.function.Consumer<TimerRefMut> action) {
        try (TimerWriteLock timerLock = write()) {
            action.accept(timerLock.timer());
        }
    }"#
        )?;
    }

    write!(
        writer,
        r#"
    {class}(int ptr) {{
        this.ptr = ptr;
    }}
}}"#,
        class = class_name_ref
    )
}

fn write_class_ref_mut<P: AsRef<Path>>(path: P, class_name: &str, class: &Class) -> Result<()> {
    let mut writer = BufWriter::new(File::create(path)?);
    let class_name_ref = format!("{}Ref", class_name);
    let class_name_ref_mut = format!("{}RefMut", class_name);

    write!(
        writer,
        r#"package org.livesplit;
"#
    )?;

    write_class_comments(&mut writer, &class.comments)?;

    write!(
        writer,
        r#"
public class {class} extends {base_class} {{"#,
        class = class_name_ref_mut,
        base_class = class_name_ref
    )?;

    for function in &class.mut_fns {
        write_fn(&mut writer, function, &class_name)?;
    }

    write!(
        writer,
        r#"
    {class}(int ptr) {{
        super(ptr);
    }}
}}"#,
        class = class_name_ref_mut
    )
}

fn write_class<P: AsRef<Path>>(path: P, class_name: &str, class: &Class) -> Result<()> {
    let mut writer = BufWriter::new(File::create(path)?);
    let class_name_ref_mut = format!("{}RefMut", class_name);

    write!(
        writer,
        r#"package org.livesplit;
"#
    )?;

    write_class_comments(&mut writer, &class.comments)?;

    write!(
        writer,
        r#"
public class {class} extends {base_class} implements AutoCloseable {{
    private void drop() {{
        if (ptr != 0) {{"#,
        class = class_name,
        base_class = class_name_ref_mut
    )?;

    if let Some(function) = class.own_fns.iter().find(|f| f.method == "drop") {
        write!(
            writer,
            r#"
            LiveSplitCoreNative.INSTANCE.{}(this.ptr);"#,
            function.name
        )?;
    }

    write!(
        writer,
        r#"
            ptr = 0;
        }}
    }}
    protected void finalize() throws Throwable {{
        drop();
        super.finalize();
    }}
    public void close() {{
        drop();
    }}"#
    )?;

    for function in class.static_fns.iter().chain(class.own_fns.iter()) {
        if function.method != "drop" {
            write_fn(&mut writer, function, &class_name)?;
        }
    }

    write!(
        writer,
        r#"
    {class}(int ptr) {{
        super(ptr);
    }}
}}"#,
        class = class_name
    )
}

fn write_native_class<P: AsRef<Path>>(path: P) -> Result<()> {
    let mut writer = BufWriter::new(File::create(path)?);

    writeln!(writer,
           "{}",
           r#"package org.livesplit;

import java.io.ByteArrayOutputStream;
import java.lang.invoke.MethodHandle;
import java.lang.invoke.MethodHandles;
import java.lang.invoke.MethodType;
import java.nio.ByteBuffer;
import java.nio.ByteOrder;
import java.nio.ShortBuffer;
import java.nio.charset.Charset;
import java.time.Instant;
import java.time.ZoneOffset;
import java.time.ZonedDateTime;

public class LiveSplitCoreNative {
    public static LiveSplitCore INSTANCE;
    static Charset charset;

    static {
        charset = Charset.forName("UTF-8");

        MethodHandle instantNow = null;
        MethodHandle dateNow = null;
        try {
            instantNow = MethodHandles.lookup().findStatic(LiveSplitCoreNative.class, "Instant_now", MethodType.methodType(double.class));
            dateNow = MethodHandles.lookup().findStatic(LiveSplitCoreNative.class, "Date_now", MethodType.methodType(void.class, int.class));
        } catch (Exception e) {}

        INSTANCE = new LiveSplitCore(10 << 20, dateNow, null, null, instantNow);
    }

    static Long firstNanoTime = null;
    static double Instant_now() {
        long nanoTime = System.nanoTime();
        if (firstNanoTime == null) {
            firstNanoTime = nanoTime;
        }
        long diff = nanoTime - firstNanoTime;
        double diffS = (double)diff / 1_000_000_000.0;
        return diffS;
    }

    static void Date_now(int ptr) {
        ZonedDateTime date = Instant.now().atZone(ZoneOffset.UTC);
        ByteBuffer mem = INSTANCE.getMemory().slice();
        ShortBuffer shorts = mem.order(ByteOrder.LITTLE_ENDIAN).asShortBuffer();
        shorts.position(ptr / 2);
        shorts.put((short)date.getYear());
        mem.position(ptr + 2);
        mem.put((byte)date.getMonth().getValue());
        mem.put((byte)date.getDayOfMonth());
        mem.put((byte)date.getHour());
        mem.put((byte)date.getMinute());
        mem.put((byte)date.getSecond());
        shorts.position((ptr / 2) + 4);
        shorts.put((short)(date.getNano() / 1_000_000));
    }

    public static class AllocatedBuf {
        public int ptr;
        public int size;

        public void dealloc() {
            INSTANCE.dealloc(ptr, size);
        }
    }

    public static AllocatedBuf allocString(String s) {
        AllocatedBuf buf = new AllocatedBuf();
        byte[] bytes = s.getBytes(charset);
        buf.size = bytes.length + 1;
        buf.ptr = INSTANCE.alloc(buf.size);
        ByteBuffer mem = INSTANCE.getMemory().slice();
        mem.position(buf.ptr);
        mem.put(bytes);
        mem.put((byte)0);
        return buf;
    }

    public static String readString(int ptr) {
        ByteBuffer mem = INSTANCE.getMemory().slice();
        mem.position(ptr);
        ByteArrayOutputStream stream = new ByteArrayOutputStream();
        while (true) {
            byte val = mem.get();
            if (val == 0) {
                break;
            }
            stream.write(val);
        }
        return new String(stream.toByteArray(), charset);
    }
}"#)
}

pub fn write<P: AsRef<Path>>(path: P, classes: &BTreeMap<String, Class>) -> Result<()> {
    let mut path = path.as_ref().to_owned();

    path.push("LiveSplitCoreNative.java");
    write_native_class(&path)?;
    path.pop();

    for (class_name, class) in classes {
        path.push(format!("{}Ref", class_name));
        path.set_extension("java");
        write_class_ref(&path, class_name, class)?;
        path.pop();

        path.push(format!("{}RefMut", class_name));
        path.set_extension("java");
        write_class_ref_mut(&path, class_name, class)?;
        path.pop();

        path.push(class_name);
        path.set_extension("java");
        write_class(&path, class_name, class)?;
        path.pop();
    }

    Ok(())
}
