//! C FFI bindings for the Bedrock protocol library.
//!
//! This crate provides a stable C API for use from C, C++, Python (via ctypes),
//! and other languages with C interop.
//!
//! # Error handling
//!
//! Functions return `i32` error codes:
//! - `0` = success
//! - Negative values = error (see `BedrockError` constants)
//!
//! On error, call `bedrock_last_error()` to get a human-readable message.
//!
//! # Memory ownership
//!
//! - Streams and packets created with `bedrock_*_create` must be freed with `bedrock_*_destroy`.
//! - Data buffers returned by `bedrock_*_serialize` must be freed with `bedrock_free`.
//! - Strings returned by `bedrock_*_get_name` are valid until the packet is destroyed.

#![allow(unused)]

use std::ffi::{CStr, CString};
use std::os::raw::c_char;
use std::sync::Mutex;

use bedrock_protocol::{
    BinaryStream, ReadOnlyBinaryStream,
    BedrockRead, BedrockWrite, BedrockSerializable, BinaryStreamError,
    Packet, UnimplementedPacket,
};

// ---------------------------------------------------------------------------
// Error codes
// ---------------------------------------------------------------------------

pub const BEDROCK_SUCCESS: i32 = 0;
pub const BEDROCK_ERR_OVERFLOW: i32 = -1;
pub const BEDROCK_ERR_INVALID_DATA: i32 = -2;
pub const BEDROCK_ERR_INVALID_ARG: i32 = -3;
pub const BEDROCK_ERR_UNSUPPORTED: i32 = -4;
pub const BEDROCK_ERR_NBT: i32 = -5;

// ---------------------------------------------------------------------------
// Thread-local error handling
// ---------------------------------------------------------------------------

std::thread_local! {
    static LAST_ERROR: Mutex<Option<CString>> = const { Mutex::new(None) };
}

fn set_error(msg: String) {
    LAST_ERROR.with(|e| {
        *e.lock().unwrap_or_else(|e| e.into_inner()) = Some(CString::new(msg).unwrap_or_default());
    });
}

fn clear_error() {
    LAST_ERROR.with(|e| {
        *e.lock().unwrap_or_else(|e| e.into_inner()) = None;
    });
}

fn to_error_code(err: BinaryStreamError) -> i32 {
    match err {
        BinaryStreamError::Overflow { .. } => BEDROCK_ERR_OVERFLOW,
        BinaryStreamError::InvalidData { .. } => BEDROCK_ERR_INVALID_DATA,
        BinaryStreamError::UnsupportedValue { .. } => BEDROCK_ERR_UNSUPPORTED,
        BinaryStreamError::NbtError { .. } => BEDROCK_ERR_NBT,
    }
}

fn handle_error(err: BinaryStreamError) -> i32 {
    set_error(err.to_string());
    to_error_code(err)
}

use std::panic::{catch_unwind, AssertUnwindSafe};

macro_rules! c_try {
    ($expr:expr) => {
        match catch_unwind(AssertUnwindSafe($expr)) {
            Ok(Ok(val)) => val,
            Ok(Err(e)) => return handle_error(e),
            Err(_) => {
                set_error("panic in Rust code".to_string());
                return BEDROCK_ERR_INVALID_DATA;
            }
        }
    };
}

// ---------------------------------------------------------------------------
// Opaque handle types
// ---------------------------------------------------------------------------

/// An opaque handle for a binary stream.
pub enum FfiStream {
    /// Read-only stream wrapping a ReadOnlyBinaryStream.
    ReadOnly(ReadOnlyBinaryStream),
    /// Read-write stream wrapping a BinaryStream.
    ReadWrite(BinaryStream),
}

/// An opaque handle for a packet.
pub struct FfiPacket {
    inner: UnimplementedPacket,
    name: CString,
}

// ---------------------------------------------------------------------------
// Stream lifecycle
// ---------------------------------------------------------------------------

/// Create a new writable binary stream.
#[no_mangle]
pub extern "C" fn bedrock_stream_create(big_endian: bool) -> *mut FfiStream {
    clear_error();
    let stream = FfiStream::ReadWrite(BinaryStream::new(big_endian));
    Box::into_raw(Box::new(stream))
}

/// Create a read-only binary stream from a byte buffer.
/// The data is copied into the stream.
#[no_mangle]
pub extern "C" fn bedrock_stream_from_bytes(
    data: *const u8,
    len: usize,
    big_endian: bool,
) -> *mut FfiStream {
    clear_error();
    if data.is_null() {
        set_error("null data pointer".to_string());
        return std::ptr::null_mut();
    }
    let slice = unsafe { std::slice::from_raw_parts(data, len) };
    let stream = FfiStream::ReadOnly(ReadOnlyBinaryStream::new(slice, big_endian));
    Box::into_raw(Box::new(stream))
}

/// Destroy a stream handle.
#[no_mangle]
pub extern "C" fn bedrock_stream_destroy(stream: *mut FfiStream) {
    if !stream.is_null() {
        unsafe { drop(Box::from_raw(stream)); }
    }
}

// ---------------------------------------------------------------------------
// Stream read operations
// ---------------------------------------------------------------------------

/// Helper: get read access to the stream.
fn with_read<F>(stream: *mut FfiStream, f: F) -> i32
where
    F: FnOnce(&mut ReadOnlyBinaryStream) -> Result<(), BinaryStreamError>,
{
    if stream.is_null() {
        set_error("null stream pointer".to_string());
        return BEDROCK_ERR_INVALID_ARG;
    }
    let stream = unsafe { &mut *stream };
    let result = match stream {
        FfiStream::ReadOnly(ref mut s) => f(s),
        FfiStream::ReadWrite(_) => {
            set_error("read not supported on write-only stream".to_string());
            return BEDROCK_ERR_UNSUPPORTED;
        }
    };
    match result {
        Ok(()) => BEDROCK_SUCCESS,
        Err(e) => handle_error(e),
    }
}

macro_rules! define_read_fn {
    ($name:ident, $method:ident, $out_type:ty) => {
        #[no_mangle]
        pub extern "C" fn $name(
            stream: *mut FfiStream,
            out: *mut $out_type,
        ) -> i32 {
            if out.is_null() {
                set_error("null out pointer".to_string());
                return BEDROCK_ERR_INVALID_ARG;
            }
            with_read(stream, |s| {
                let val = s.$method()?;
                unsafe { *out = val; }
                Ok(())
            })
        }
    };
}

define_read_fn!(bedrock_stream_read_bool, read_bool, bool);
define_read_fn!(bedrock_stream_read_u8, read_u8, u8);
define_read_fn!(bedrock_stream_read_i16, read_i16, i16);
define_read_fn!(bedrock_stream_read_u16, read_u16, u16);
define_read_fn!(bedrock_stream_read_i32, read_i32, i32);
define_read_fn!(bedrock_stream_read_u32, read_u32, u32);
define_read_fn!(bedrock_stream_read_i64, read_i64, i64);
define_read_fn!(bedrock_stream_read_u64, read_u64, u64);
define_read_fn!(bedrock_stream_read_f32, read_f32, f32);
define_read_fn!(bedrock_stream_read_f64, read_f64, f64);
define_read_fn!(bedrock_stream_read_i32_be, read_i32_be, i32);
define_read_fn!(bedrock_stream_read_u32_be, read_u32_be, u32);
define_read_fn!(bedrock_stream_read_u24, read_u24, u32);
define_read_fn!(bedrock_stream_read_varint, read_varint, i32);
define_read_fn!(bedrock_stream_read_varint64, read_varint64, i64);
define_read_fn!(bedrock_stream_read_unsigned_varint, read_unsigned_varint, u32);
define_read_fn!(bedrock_stream_read_unsigned_varint64, read_unsigned_varint64, u64);
define_read_fn!(bedrock_stream_read_normalized_f32, read_normalized_f32, f32);

/// Read a string from the stream into a caller-provided buffer.
/// `inout_len` must point to the buffer size on input; on return it contains
/// the actual string length (including null terminator).
#[no_mangle]
pub extern "C" fn bedrock_stream_read_string(
    stream: *mut FfiStream,
    buffer: *mut c_char,
    inout_len: *mut usize,
) -> i32 {
    if inout_len.is_null() {
        set_error("null inout_len pointer".to_string());
        return BEDROCK_ERR_INVALID_ARG;
    }
    let max_len = unsafe { *inout_len };

    // First read the string to know its size
    let result = catch_unwind(AssertUnwindSafe(|| {
        let s = unsafe { &mut *stream };
        match s {
            FfiStream::ReadOnly(ref mut s) => s.read_string(),
            FfiStream::ReadWrite(_) => Err(BinaryStreamError::UnsupportedValue {
                description: "read not supported on write stream".to_string(),
            }),
        }
    }));

    let string = match result {
        Ok(Ok(s)) => s,
        Ok(Err(e)) => return handle_error(e),
        Err(_) => {
            set_error("panic in Rust code".to_string());
            return BEDROCK_ERR_INVALID_DATA;
        }
    };

    let needed = string.len() + 1; // +1 for null terminator

    if buffer.is_null() || max_len < needed {
        // Caller needs more space — tell them how much
        unsafe { *inout_len = needed; }
        return if buffer.is_null() { BEDROCK_SUCCESS } else { BEDROCK_ERR_INVALID_ARG };
    }

    unsafe {
        std::ptr::copy_nonoverlapping(string.as_ptr() as *const c_char, buffer, string.len());
        *buffer.add(string.len()) = 0; // null terminator
        *inout_len = needed;
    }
    BEDROCK_SUCCESS
}

// ---------------------------------------------------------------------------
// Stream write operations
// ---------------------------------------------------------------------------

/// Helper: get write access to the stream.
fn with_write<F>(stream: *mut FfiStream, f: F) -> i32
where
    F: FnOnce(&mut BinaryStream) -> Result<(), BinaryStreamError>,
{
    if stream.is_null() {
        set_error("null stream pointer".to_string());
        return BEDROCK_ERR_INVALID_ARG;
    }
    let stream = unsafe { &mut *stream };
    match stream {
        FfiStream::ReadOnly(_) => {
            set_error("cannot write to a read-only stream".to_string());
            BEDROCK_ERR_INVALID_ARG
        }
        FfiStream::ReadWrite(ref mut s) => match f(s) {
            Ok(()) => BEDROCK_SUCCESS,
            Err(e) => handle_error(e),
        },
    }
}

macro_rules! define_write_fn {
    ($name:ident, $method:ident, $in_type:ty) => {
        #[no_mangle]
        pub extern "C" fn $name(
            stream: *mut FfiStream,
            value: $in_type,
        ) -> i32 {
            with_write(stream, |s| s.$method(value))
        }
    };
}

define_write_fn!(bedrock_stream_write_bool, write_bool, bool);
define_write_fn!(bedrock_stream_write_u8, write_u8, u8);
define_write_fn!(bedrock_stream_write_i16, write_i16, i16);
define_write_fn!(bedrock_stream_write_u16, write_u16, u16);
define_write_fn!(bedrock_stream_write_i32, write_i32, i32);
define_write_fn!(bedrock_stream_write_u32, write_u32, u32);
define_write_fn!(bedrock_stream_write_i64, write_i64, i64);
define_write_fn!(bedrock_stream_write_u64, write_u64, u64);
define_write_fn!(bedrock_stream_write_f32, write_f32, f32);
define_write_fn!(bedrock_stream_write_f64, write_f64, f64);
define_write_fn!(bedrock_stream_write_i32_be, write_i32_be, i32);
define_write_fn!(bedrock_stream_write_u32_be, write_u32_be, u32);
define_write_fn!(bedrock_stream_write_u24, write_u24, u32);
define_write_fn!(bedrock_stream_write_varint, write_varint, i32);
define_write_fn!(bedrock_stream_write_varint64, write_varint64, i64);
define_write_fn!(bedrock_stream_write_unsigned_varint, write_unsigned_varint, u32);
define_write_fn!(bedrock_stream_write_unsigned_varint64, write_unsigned_varint64, u64);
define_write_fn!(bedrock_stream_write_normalized_f32, write_normalized_f32, f32);

/// Write a string to the stream.
#[no_mangle]
pub extern "C" fn bedrock_stream_write_string(
    stream: *mut FfiStream,
    value: *const c_char,
) -> i32 {
    if value.is_null() {
        set_error("null string pointer".to_string());
        return BEDROCK_ERR_INVALID_ARG;
    }
    let c_str = unsafe { CStr::from_ptr(value) };
    let rust_str = match c_str.to_str() {
        Ok(s) => s,
        Err(_) => {
            set_error("invalid UTF-8 in string".to_string());
            return BEDROCK_ERR_INVALID_DATA;
        }
    };
    with_write(stream, |s| s.write_string(rust_str))
}

/// Write raw bytes to the stream.
#[no_mangle]
pub extern "C" fn bedrock_stream_write_raw_bytes(
    stream: *mut FfiStream,
    data: *const u8,
    len: usize,
) -> i32 {
    if data.is_null() {
        set_error("null data pointer".to_string());
        return BEDROCK_ERR_INVALID_ARG;
    }
    let slice = unsafe { std::slice::from_raw_parts(data, len) };
    with_write(stream, |s| s.write_raw_bytes(slice))
}

/// Read raw bytes from the stream into a caller-provided buffer.
#[no_mangle]
pub extern "C" fn bedrock_stream_read_raw_bytes(
    stream: *mut FfiStream,
    buffer: *mut u8,
    len: usize,
) -> i32 {
    if buffer.is_null() {
        set_error("null buffer pointer".to_string());
        return BEDROCK_ERR_INVALID_ARG;
    }
    with_read(stream, |s| {
        let data = s.read_raw_bytes(len)?;
        unsafe { std::ptr::copy_nonoverlapping(data.as_ptr(), buffer, len); }
        Ok(())
    })
}

// ---------------------------------------------------------------------------
// Stream utility
// ---------------------------------------------------------------------------

#[no_mangle]
pub extern "C" fn bedrock_stream_size(stream: *const FfiStream) -> usize {
    if stream.is_null() { return 0; }
    let stream = unsafe { &*stream };
    match stream {
        FfiStream::ReadOnly(s) => s.size(),
        FfiStream::ReadWrite(s) => s.position(),
    }
}

#[no_mangle]
pub extern "C" fn bedrock_stream_position(stream: *const FfiStream) -> usize {
    if stream.is_null() { return 0; }
    let stream = unsafe { &*stream };
    match stream {
        FfiStream::ReadOnly(s) => s.position(),
        FfiStream::ReadWrite(s) => s.position(),
    }
}

#[no_mangle]
pub extern "C" fn bedrock_stream_set_position(
    stream: *mut FfiStream,
    pos: usize,
) -> i32 {
    if stream.is_null() {
        set_error("null stream pointer".to_string());
        return BEDROCK_ERR_INVALID_ARG;
    }
    let stream = unsafe { &mut *stream };
    match stream {
        FfiStream::ReadOnly(s) => match s.set_position(pos) {
            Ok(()) => BEDROCK_SUCCESS,
            Err(e) => handle_error(e),
        },
        FfiStream::ReadWrite(_) => {
            set_error("set_position not supported on write streams".to_string());
            BEDROCK_ERR_UNSUPPORTED
        }
    }
}

/// Get a pointer to the stream's internal data buffer.
/// The returned pointer is valid until the stream is destroyed or modified.
#[no_mangle]
pub extern "C" fn bedrock_stream_data(
    stream: *const FfiStream,
    out_data: *mut *const u8,
    out_len: *mut usize,
) -> i32 {
    if out_data.is_null() || out_len.is_null() {
        set_error("null out pointer".to_string());
        return BEDROCK_ERR_INVALID_ARG;
    }
    if stream.is_null() {
        set_error("null stream pointer".to_string());
        return BEDROCK_ERR_INVALID_ARG;
    }
    let stream = unsafe { &*stream };
    let (data, len) = match stream {
        FfiStream::ReadOnly(s) => (s.as_slice().as_ptr(), s.size()),
        FfiStream::ReadWrite(s) => (s.as_slice().as_ptr(), s.as_slice().len()),
    };
    unsafe {
        *out_data = data;
        *out_len = len;
    }
    BEDROCK_SUCCESS
}

// ---------------------------------------------------------------------------
// NBT operations (wraps bedrock-nbt CompoundTag for Python FFI)
// ---------------------------------------------------------------------------

use bedrock_nbt::CompoundTag as NbtCompoundTag;

/// Opaque handle for an NBT CompoundTag.
pub struct FfiNbt {
    inner: NbtCompoundTag,
}

/// Create a new empty CompoundTag.
#[no_mangle]
pub extern "C" fn bedrock_nbt_create() -> *mut FfiNbt {
    clear_error();
    Box::into_raw(Box::new(FfiNbt { inner: NbtCompoundTag::new() }))
}

/// Destroy an NBT handle.
#[no_mangle]
pub extern "C" fn bedrock_nbt_destroy(nbt: *mut FfiNbt) {
    if !nbt.is_null() {
        unsafe { drop(Box::from_raw(nbt)); }
    }
}

/// Set a string value.
#[no_mangle]
pub extern "C" fn bedrock_nbt_set_string(
    nbt: *mut FfiNbt,
    key: *const c_char,
    value: *const c_char,
) -> i32 {
    if nbt.is_null() || key.is_null() || value.is_null() {
        return BEDROCK_ERR_INVALID_ARG;
    }
    let key = unsafe { CStr::from_ptr(key) }.to_str().unwrap_or("");
    let value = unsafe { CStr::from_ptr(value) }.to_str().unwrap_or("");
    let nbt = unsafe { &mut *nbt };
    nbt.inner.set(key, value.to_string());
    BEDROCK_SUCCESS
}

/// Set an integer value.
#[no_mangle]
pub extern "C" fn bedrock_nbt_set_int(
    nbt: *mut FfiNbt,
    key: *const c_char,
    value: i32,
) -> i32 {
    if nbt.is_null() || key.is_null() {
        return BEDROCK_ERR_INVALID_ARG;
    }
    let key = unsafe { CStr::from_ptr(key) }.to_str().unwrap_or("");
    let nbt = unsafe { &mut *nbt };
    nbt.inner.set(key, value);
    BEDROCK_SUCCESS
}

/// Set a short (int16) value.
#[no_mangle]
pub extern "C" fn bedrock_nbt_set_short(
    nbt: *mut FfiNbt,
    key: *const c_char,
    value: i16,
) -> i32 {
    if nbt.is_null() || key.is_null() {
        return BEDROCK_ERR_INVALID_ARG;
    }
    let key = unsafe { CStr::from_ptr(key) }.to_str().unwrap_or("");
    let nbt = unsafe { &mut *nbt };
    nbt.inner.set(key, value);
    BEDROCK_SUCCESS
}

/// Set a byte (bool/int8) value.
#[no_mangle]
pub extern "C" fn bedrock_nbt_set_byte(
    nbt: *mut FfiNbt,
    key: *const c_char,
    value: i8,
) -> i32 {
    if nbt.is_null() || key.is_null() {
        return BEDROCK_ERR_INVALID_ARG;
    }
    let key = unsafe { CStr::from_ptr(key) }.to_str().unwrap_or("");
    let nbt = unsafe { &mut *nbt };
    nbt.inner.set(key, value);
    BEDROCK_SUCCESS
}

/// Append a string to a list within the compound (creates list if not exists).
#[no_mangle]
pub extern "C" fn bedrock_nbt_list_append_string(
    nbt: *mut FfiNbt,
    list_key: *const c_char,
    value: *const c_char,
) -> i32 {
    if nbt.is_null() || list_key.is_null() || value.is_null() {
        return BEDROCK_ERR_INVALID_ARG;
    }
    let list_key = unsafe { CStr::from_ptr(list_key) }.to_str().unwrap_or("");
    let value = unsafe { CStr::from_ptr(value) }.to_str().unwrap_or("");
    let nbt = unsafe { &mut *nbt };

    match nbt.inner.get_mut(list_key) {
        Some(bedrock_nbt::Tag::List(ref mut lst)) => {
            lst.elements.push(bedrock_nbt::Tag::String(value.to_string()));
        }
        _ => {
            let mut new_list = bedrock_nbt::ListTag::new();
            new_list.append(value.to_string());
            nbt.inner.set(list_key, new_list);
        }
    }
    BEDROCK_SUCCESS
}

/// Serialize the CompoundTag to Little Endian binary NBT.
/// Returns bytes that must be freed with bedrock_free.
#[no_mangle]
pub extern "C" fn bedrock_nbt_to_binary(
    nbt: *mut FfiNbt,
    out_data: *mut *mut u8,
    out_len: *mut usize,
) -> i32 {
    if out_data.is_null() || out_len.is_null() {
        return BEDROCK_ERR_INVALID_ARG;
    }
    if nbt.is_null() {
        return BEDROCK_ERR_INVALID_ARG;
    }
    let nbt = unsafe { &*nbt };
    let bytes = nbt.inner.to_binary_nbt(true, false);
    let len = bytes.len();
    let buf = unsafe { libc::malloc(len) as *mut u8 };
    if buf.is_null() {
        set_error("memory allocation failed".to_string());
        return BEDROCK_ERR_UNSUPPORTED;
    }
    unsafe {
        std::ptr::copy_nonoverlapping(bytes.as_ptr(), buf, len);
        *out_data = buf;
        *out_len = len;
    }
    BEDROCK_SUCCESS
}

/// Serialize to Bedrock Network NBT format.
#[no_mangle]
pub extern "C" fn bedrock_nbt_to_network(
    nbt: *mut FfiNbt,
    out_data: *mut *mut u8,
    out_len: *mut usize,
) -> i32 {
    if out_data.is_null() || out_len.is_null() {
        return BEDROCK_ERR_INVALID_ARG;
    }
    if nbt.is_null() {
        return BEDROCK_ERR_INVALID_ARG;
    }
    let nbt = unsafe { &*nbt };
    let bytes = nbt.inner.to_network_nbt();
    let len = bytes.len();
    let buf = unsafe { libc::malloc(len) as *mut u8 };
    if buf.is_null() {
        set_error("memory allocation failed".to_string());
        return BEDROCK_ERR_UNSUPPORTED;
    }
    unsafe {
        std::ptr::copy_nonoverlapping(bytes.as_ptr(), buf, len);
        *out_data = buf;
        *out_len = len;
    }
    BEDROCK_SUCCESS
}

/// Check if the compound is empty.
#[no_mangle]
pub extern "C" fn bedrock_nbt_empty(nbt: *const FfiNbt) -> bool {
    if nbt.is_null() { return true; }
    let nbt = unsafe { &*nbt };
    nbt.inner.empty()
}

/// Check if a key exists.
#[no_mangle]
pub extern "C" fn bedrock_nbt_contains(
    nbt: *const FfiNbt,
    key: *const c_char,
) -> bool {
    if nbt.is_null() || key.is_null() { return false; }
    let key = unsafe { CStr::from_ptr(key) }.to_str().unwrap_or("");
    let nbt = unsafe { &*nbt };
    nbt.inner.contains(key)
}

/// Set a CompoundTag field within a parent CompoundTag.
/// Takes ownership of the child — the child handle is consumed on success.
#[no_mangle]
pub extern "C" fn bedrock_nbt_set_tag(
    nbt: *mut FfiNbt,
    key: *const c_char,
    child: *mut FfiNbt,
) -> i32 {
    if nbt.is_null() || key.is_null() || child.is_null() {
        return BEDROCK_ERR_INVALID_ARG;
    }
    let key_str = unsafe { CStr::from_ptr(key) }.to_str().unwrap_or("").to_string();
    let nbt = unsafe { &mut *nbt };
    let child = unsafe { Box::from_raw(child) };
    nbt.inner.set(&key_str, child.inner);
    BEDROCK_SUCCESS
}

/// Append a CompoundTag to a list within the compound.
/// Takes ownership of the child handle.
#[no_mangle]
pub extern "C" fn bedrock_nbt_list_append_tag(
    nbt: *mut FfiNbt,
    list_key: *const c_char,
    child: *mut FfiNbt,
) -> i32 {
    if nbt.is_null() || list_key.is_null() || child.is_null() {
        return BEDROCK_ERR_INVALID_ARG;
    }
    let list_key_str = unsafe { CStr::from_ptr(list_key) }.to_str().unwrap_or("").to_string();
    let nbt = unsafe { &mut *nbt };
    let child = unsafe { Box::from_raw(child) };
    match nbt.inner.get_mut(&list_key_str) {
        Some(bedrock_nbt::Tag::List(ref mut lst)) => {
            lst.elements.push(child.inner.to_tag());
        }
        _ => {
            let mut new_list = bedrock_nbt::ListTag::new();
            new_list.append(child.inner);
            nbt.inner.set(&list_key_str, new_list.to_tag());
        }
    }
    BEDROCK_SUCCESS
}

/// Write the CompoundTag to a BinaryStream in Network NBT format.
#[no_mangle]
pub extern "C" fn bedrock_nbt_write_to_stream(
    nbt: *const FfiNbt,
    stream: *mut FfiStream,
) -> i32 {
    if nbt.is_null() || stream.is_null() {
        return BEDROCK_ERR_INVALID_ARG;
    }
    let nbt = unsafe { &*nbt };
    let bytes = nbt.inner.to_network_nbt();
    with_write(stream, |s| s.write_raw_bytes(&bytes))
}

/// Get an integer value from a compound tag.
#[no_mangle]
pub extern "C" fn bedrock_nbt_get_int(
    nbt: *const FfiNbt,
    key: *const c_char,
    out: *mut i32,
) -> i32 {
    if nbt.is_null() || key.is_null() || out.is_null() {
        return BEDROCK_ERR_INVALID_ARG;
    }
    let key = unsafe { CStr::from_ptr(key) }.to_str().unwrap_or("");
    let nbt = unsafe { &*nbt };
    match nbt.inner.get(key) {
        Some(bedrock_nbt::Tag::Int(v)) => {
            unsafe { *out = *v; }
            BEDROCK_SUCCESS
        }
        Some(bedrock_nbt::Tag::Short(v)) => {
            unsafe { *out = *v as i32; }
            BEDROCK_SUCCESS
        }
        Some(bedrock_nbt::Tag::Byte(v)) => {
            unsafe { *out = *v as i32; }
            BEDROCK_SUCCESS
        }
        _ => BEDROCK_ERR_INVALID_DATA,
    }
}

/// Get a string value from a compound tag. Returns the string in a caller-provided buffer.
#[no_mangle]
pub extern "C" fn bedrock_nbt_get_string(
    nbt: *const FfiNbt,
    key: *const c_char,
    buffer: *mut c_char,
    inout_len: *mut usize,
) -> i32 {
    if nbt.is_null() || key.is_null() || inout_len.is_null() {
        return BEDROCK_ERR_INVALID_ARG;
    }
    let key = unsafe { CStr::from_ptr(key) }.to_str().unwrap_or("");
    let nbt = unsafe { &*nbt };
    let value = match nbt.inner.get(key) {
        Some(bedrock_nbt::Tag::String(s)) => s.clone(),
        _ => return BEDROCK_ERR_INVALID_DATA,
    };
    let needed = value.len() + 1;
    if buffer.is_null() || unsafe { *inout_len } < needed {
        unsafe { *inout_len = needed; }
        return if buffer.is_null() { BEDROCK_SUCCESS } else { BEDROCK_ERR_INVALID_ARG };
    }
    unsafe {
        std::ptr::copy_nonoverlapping(value.as_ptr() as *const c_char, buffer, value.len());
        *buffer.add(value.len()) = 0;
        *inout_len = needed;
    }
    BEDROCK_SUCCESS
}

/// Serialize to SNBT (stringified NBT) for debugging.
/// Returns a C string that must be freed with bedrock_free.
#[no_mangle]
pub extern "C" fn bedrock_nbt_to_snbt(
    nbt: *const FfiNbt,
) -> *mut c_char {
    if nbt.is_null() { return std::ptr::null_mut(); }
    clear_error();
    let nbt = unsafe { &*nbt };
    let snbt = nbt.inner.to_snbt();
    let len = snbt.len();
    let buf = unsafe { libc::malloc(len + 1) as *mut c_char };
    if buf.is_null() {
        set_error("memory allocation failed".to_string());
        return std::ptr::null_mut();
    }
    unsafe {
        std::ptr::copy_nonoverlapping(snbt.as_ptr() as *const c_char, buf, len);
        *buf.add(len) = 0; // null terminator
    }
    buf
}

/// Parse a CompoundTag from Network NBT bytes into a pre-allocated handle.
/// Writes the number of bytes consumed to `out_consumed` on success.
/// The `nbt` handle must have been created with bedrock_nbt_create.
/// Returns 0 on success, negative on error.
#[no_mangle]
pub extern "C" fn bedrock_nbt_from_network_into(
    nbt: *mut FfiNbt,
    data: *const u8,
    len: usize,
    out_consumed: *mut usize,
) -> i32 {
    clear_error();
    if nbt.is_null() || data.is_null() || out_consumed.is_null() {
        set_error("null pointer".to_string());
        return BEDROCK_ERR_INVALID_ARG;
    }
    let slice = unsafe { std::slice::from_raw_parts(data, len) };
    let nbt = unsafe { &mut *nbt };
    match bedrock_nbt::decode::from_network_nbt(slice) {
        Ok((tag, consumed)) => {
            nbt.inner = tag;
            unsafe { *out_consumed = consumed; }
            BEDROCK_SUCCESS
        }
        Err(e) => {
            set_error(e.to_string());
            BEDROCK_ERR_INVALID_DATA
        }
    }
}

// ---------------------------------------------------------------------------
// Packet operations
// ---------------------------------------------------------------------------

/// Create a packet by numeric ID.
#[no_mangle]
pub extern "C" fn bedrock_packet_create(packet_id: u32) -> *mut FfiPacket {
    clear_error();
    let inner = UnimplementedPacket::new(packet_id as i32);
    let name = CString::new(Packet::packet_name(&inner)).unwrap_or_default();
    Box::into_raw(Box::new(FfiPacket { inner, name }))
}

/// Destroy a packet handle.
#[no_mangle]
pub extern "C" fn bedrock_packet_destroy(packet: *mut FfiPacket) {
    if !packet.is_null() {
        unsafe { drop(Box::from_raw(packet)); }
    }
}

/// Serialize a packet to bytes. The caller must free the returned buffer with `bedrock_free`.
#[no_mangle]
pub extern "C" fn bedrock_packet_serialize(
    packet: *mut FfiPacket,
    out_data: *mut *mut u8,
    out_len: *mut usize,
) -> i32 {
    if out_data.is_null() || out_len.is_null() {
        set_error("null out pointer".to_string());
        return BEDROCK_ERR_INVALID_ARG;
    }
    if packet.is_null() {
        set_error("null packet pointer".to_string());
        return BEDROCK_ERR_INVALID_ARG;
    }
    let packet = unsafe { &*packet };
    let result = catch_unwind(AssertUnwindSafe(|| Packet::serialize(&packet.inner)));
    let bytes = match result {
        Ok(Ok(b)) => b,
        Ok(Err(e)) => return handle_error(e),
        Err(_) => {
            set_error("panic during serialization".to_string());
            return BEDROCK_ERR_INVALID_DATA;
        }
    };
    let len = bytes.len();
    let buf = unsafe { libc::malloc(len) as *mut u8 };
    if buf.is_null() {
        set_error("memory allocation failed".to_string());
        return BEDROCK_ERR_UNSUPPORTED;
    }
    unsafe {
        std::ptr::copy_nonoverlapping(bytes.as_ptr(), buf, len);
        *out_data = buf;
        *out_len = len;
    }
    BEDROCK_SUCCESS
}

/// Deserialize a packet from bytes.
#[no_mangle]
pub extern "C" fn bedrock_packet_deserialize(
    packet: *mut FfiPacket,
    data: *const u8,
    len: usize,
) -> i32 {
    if packet.is_null() {
        set_error("null packet pointer".to_string());
        return BEDROCK_ERR_INVALID_ARG;
    }
    if data.is_null() {
        set_error("null data pointer".to_string());
        return BEDROCK_ERR_INVALID_ARG;
    }
    let packet = unsafe { &mut *packet };
    let slice = unsafe { std::slice::from_raw_parts(data, len) };
    let result = catch_unwind(AssertUnwindSafe(|| Packet::deserialize(&mut packet.inner, slice)));
    match result {
        Ok(Ok(())) => {
            packet.name = CString::new(Packet::packet_name(&packet.inner)).unwrap_or_default();
            BEDROCK_SUCCESS
        }
        Ok(Err(e)) => handle_error(e),
        Err(_) => {
            set_error("panic during deserialization".to_string());
            BEDROCK_ERR_INVALID_DATA
        }
    }
}

/// Get the numeric ID of a packet.
#[no_mangle]
pub extern "C" fn bedrock_packet_get_id(packet: *const FfiPacket) -> u32 {
    if packet.is_null() { return u32::MAX; }
    let packet = unsafe { &*packet };
    Packet::packet_id(&packet.inner) as u32
}

/// Get the name of a packet (valid until the packet is destroyed).
#[no_mangle]
pub extern "C" fn bedrock_packet_get_name(packet: *const FfiPacket) -> *const c_char {
    if packet.is_null() { return std::ptr::null(); }
    let packet = unsafe { &*packet };
    packet.name.as_ptr() as *const c_char
}

// ---------------------------------------------------------------------------
// BlockPos operations
// ---------------------------------------------------------------------------

use bedrock_protocol::types::BlockPos;

/// Opaque handle for a BlockPos.
pub struct FfiBlockPos {
    inner: BlockPos,
}

/// Create a new BlockPos.
#[no_mangle]
pub extern "C" fn bedrock_block_pos_create(x: i32, y: i32, z: i32) -> *mut FfiBlockPos {
    clear_error();
    Box::into_raw(Box::new(FfiBlockPos { inner: BlockPos { x, y, z } }))
}

/// Destroy a BlockPos handle.
#[no_mangle]
pub extern "C" fn bedrock_block_pos_destroy(pos: *mut FfiBlockPos) {
    if !pos.is_null() {
        unsafe { drop(Box::from_raw(pos)); }
    }
}

/// Get the x coordinate of a BlockPos.
#[no_mangle]
pub extern "C" fn bedrock_block_pos_get_x(pos: *const FfiBlockPos) -> i32 {
    if pos.is_null() { return 0; }
    let pos = unsafe { &*pos };
    pos.inner.x
}

/// Get the y coordinate of a BlockPos.
#[no_mangle]
pub extern "C" fn bedrock_block_pos_get_y(pos: *const FfiBlockPos) -> i32 {
    if pos.is_null() { return 0; }
    let pos = unsafe { &*pos };
    pos.inner.y
}

/// Get the z coordinate of a BlockPos.
#[no_mangle]
pub extern "C" fn bedrock_block_pos_get_z(pos: *const FfiBlockPos) -> i32 {
    if pos.is_null() { return 0; }
    let pos = unsafe { &*pos };
    pos.inner.z
}

// ---------------------------------------------------------------------------
// Memory management
// ---------------------------------------------------------------------------

/// Free a buffer previously returned by bedrock_packet_serialize.
#[no_mangle]
pub extern "C" fn bedrock_free(ptr: *mut std::ffi::c_void) {
    if !ptr.is_null() {
        unsafe { libc::free(ptr); }
    }
}

// ---------------------------------------------------------------------------
// Error handling
// ---------------------------------------------------------------------------

/// Get the last error message. Returns null if no error occurred.
/// The returned string is valid until the next FFI call.
#[no_mangle]
pub extern "C" fn bedrock_last_error() -> *const c_char {
    LAST_ERROR.with(|e| {
        e.lock().unwrap_or_else(|e| e.into_inner()).as_ref()
            .map(|s| s.as_ptr() as *const c_char)
            .unwrap_or(std::ptr::null())
    })
}
