#[cfg(all(target_arch = "aarch64", feature = "aarch64_simd"))]
use core::arch::aarch64;
#[cfg(all(target_arch = "wasm32", feature = "wasm_simd"))]
use core::arch::wasm32;
#[cfg(target_arch = "x86")]
use core::arch::x86;
#[cfg(target_arch = "x86_64")]
use core::arch::x86_64;

/// Marker trait for castable data.
///
/// This trait is important to maintain the defined integrity of casting
/// against buffers of bytes. There are lots of opportunity for undefined
/// behavior to happen. By implementing an object as `Castable`, you are
/// declaring the following:
///
/// * The type is inhabited (e.g., no [Infallible](std::convert::Infallible)).
/// * The type allows any bit pattern (e.g., no `bool` or `char`).
/// * The type does not contain any padding bytes.
/// * The type's members are also `Castable`.
/// * The type is `#[repr(C)]`, `#[repr(transparent)]`, `#[repr(packed)]` or `#[repr(align)]`.
///
/// If you've used the [bytemuck](https://crates.io/crate/bytemuck) library,
/// these rules will probably seem familiar. You can automatically guarantee these
/// traits of your data with [the Castable derive macro](pkbuffer_derive::Castable).
pub unsafe trait Castable {}

unsafe impl Castable for () {}
unsafe impl Castable for u8 {}
unsafe impl Castable for i8 {}
unsafe impl Castable for u16 {}
unsafe impl Castable for i16 {}
unsafe impl Castable for u32 {}
unsafe impl Castable for i32 {}
unsafe impl Castable for u64 {}
unsafe impl Castable for i64 {}
unsafe impl Castable for usize {}
unsafe impl Castable for isize {}
unsafe impl Castable for u128 {}
unsafe impl Castable for i128 {}
unsafe impl Castable for f32 {}
unsafe impl Castable for f64 {}
unsafe impl<T: Castable> Castable for std::num::Wrapping<T> {}

unsafe impl<T: Castable> Castable for std::marker::PhantomData<T> {}
unsafe impl Castable for std::marker::PhantomPinned {}
unsafe impl<T: Castable> Castable for std::mem::ManuallyDrop<T> {}

unsafe impl<T, const N: usize> Castable for [T; N] where T: Castable {}

#[cfg(all(target_arch = "wasm32", feature = "wasm_simd"))]
unsafe impl Castable for wasm32::v128 {}

#[cfg(all(target_arch = "aarch64", feature = "aarch64_simd"))]
unsafe impl Castable for aarch64::float32x2_t {}
#[cfg(all(target_arch = "aarch64", feature = "aarch64_simd"))]
unsafe impl Castable for aarch64::float32x2x2_t {}
#[cfg(all(target_arch = "aarch64", feature = "aarch64_simd"))]
unsafe impl Castable for aarch64::float32x2x3_t {}
#[cfg(all(target_arch = "aarch64", feature = "aarch64_simd"))]
unsafe impl Castable for aarch64::float32x2x4_t {}
#[cfg(all(target_arch = "aarch64", feature = "aarch64_simd"))]
unsafe impl Castable for aarch64::float32x4_t {}
#[cfg(all(target_arch = "aarch64", feature = "aarch64_simd"))]
unsafe impl Castable for aarch64::float32x4x2_t {}
#[cfg(all(target_arch = "aarch64", feature = "aarch64_simd"))]
unsafe impl Castable for aarch64::float32x4x3_t {}
#[cfg(all(target_arch = "aarch64", feature = "aarch64_simd"))]
unsafe impl Castable for aarch64::float32x4x4_t {}
#[cfg(all(target_arch = "aarch64", feature = "aarch64_simd"))]
unsafe impl Castable for aarch64::float64x1_t {}
#[cfg(all(target_arch = "aarch64", feature = "aarch64_simd"))]
unsafe impl Castable for aarch64::float64x1x2_t {}
#[cfg(all(target_arch = "aarch64", feature = "aarch64_simd"))]
unsafe impl Castable for aarch64::float64x1x3_t {}
#[cfg(all(target_arch = "aarch64", feature = "aarch64_simd"))]
unsafe impl Castable for aarch64::float64x1x4_t {}
#[cfg(all(target_arch = "aarch64", feature = "aarch64_simd"))]
unsafe impl Castable for aarch64::float64x2_t {}
#[cfg(all(target_arch = "aarch64", feature = "aarch64_simd"))]
unsafe impl Castable for aarch64::float64x2x2_t {}
#[cfg(all(target_arch = "aarch64", feature = "aarch64_simd"))]
unsafe impl Castable for aarch64::float64x2x3_t {}
#[cfg(all(target_arch = "aarch64", feature = "aarch64_simd"))]
unsafe impl Castable for aarch64::float64x2x4_t {}
#[cfg(all(target_arch = "aarch64", feature = "aarch64_simd"))]
unsafe impl Castable for aarch64::int16x4_t {}
#[cfg(all(target_arch = "aarch64", feature = "aarch64_simd"))]
unsafe impl Castable for aarch64::int16x4x2_t {}
#[cfg(all(target_arch = "aarch64", feature = "aarch64_simd"))]
unsafe impl Castable for aarch64::int16x4x3_t {}
#[cfg(all(target_arch = "aarch64", feature = "aarch64_simd"))]
unsafe impl Castable for aarch64::int16x4x4_t {}
#[cfg(all(target_arch = "aarch64", feature = "aarch64_simd"))]
unsafe impl Castable for aarch64::int16x8_t {}
#[cfg(all(target_arch = "aarch64", feature = "aarch64_simd"))]
unsafe impl Castable for aarch64::int16x8x2_t {}
#[cfg(all(target_arch = "aarch64", feature = "aarch64_simd"))]
unsafe impl Castable for aarch64::int16x8x3_t {}
#[cfg(all(target_arch = "aarch64", feature = "aarch64_simd"))]
unsafe impl Castable for aarch64::int16x8x4_t {}
#[cfg(all(target_arch = "aarch64", feature = "aarch64_simd"))]
unsafe impl Castable for aarch64::int32x2_t {}
#[cfg(all(target_arch = "aarch64", feature = "aarch64_simd"))]
unsafe impl Castable for aarch64::int32x2x2_t {}
#[cfg(all(target_arch = "aarch64", feature = "aarch64_simd"))]
unsafe impl Castable for aarch64::int32x2x3_t {}
#[cfg(all(target_arch = "aarch64", feature = "aarch64_simd"))]
unsafe impl Castable for aarch64::int32x2x4_t {}
#[cfg(all(target_arch = "aarch64", feature = "aarch64_simd"))]
unsafe impl Castable for aarch64::int32x4_t {}
#[cfg(all(target_arch = "aarch64", feature = "aarch64_simd"))]
unsafe impl Castable for aarch64::int32x4x2_t {}
#[cfg(all(target_arch = "aarch64", feature = "aarch64_simd"))]
unsafe impl Castable for aarch64::int32x4x3_t {}
#[cfg(all(target_arch = "aarch64", feature = "aarch64_simd"))]
unsafe impl Castable for aarch64::int32x4x4_t {}
#[cfg(all(target_arch = "aarch64", feature = "aarch64_simd"))]
unsafe impl Castable for aarch64::int64x1_t {}
#[cfg(all(target_arch = "aarch64", feature = "aarch64_simd"))]
unsafe impl Castable for aarch64::int64x1x2_t {}
#[cfg(all(target_arch = "aarch64", feature = "aarch64_simd"))]
unsafe impl Castable for aarch64::int64x1x3_t {}
#[cfg(all(target_arch = "aarch64", feature = "aarch64_simd"))]
unsafe impl Castable for aarch64::int64x1x4_t {}
#[cfg(all(target_arch = "aarch64", feature = "aarch64_simd"))]
unsafe impl Castable for aarch64::int64x2_t {}
#[cfg(all(target_arch = "aarch64", feature = "aarch64_simd"))]
unsafe impl Castable for aarch64::int64x2x2_t {}
#[cfg(all(target_arch = "aarch64", feature = "aarch64_simd"))]
unsafe impl Castable for aarch64::int64x2x3_t {}
#[cfg(all(target_arch = "aarch64", feature = "aarch64_simd"))]
unsafe impl Castable for aarch64::int64x2x4_t {}
#[cfg(all(target_arch = "aarch64", feature = "aarch64_simd"))]
unsafe impl Castable for aarch64::int8x16_t {}
#[cfg(all(target_arch = "aarch64", feature = "aarch64_simd"))]
unsafe impl Castable for aarch64::int8x16x2_t {}
#[cfg(all(target_arch = "aarch64", feature = "aarch64_simd"))]
unsafe impl Castable for aarch64::int8x16x3_t {}
#[cfg(all(target_arch = "aarch64", feature = "aarch64_simd"))]
unsafe impl Castable for aarch64::int8x16x4_t {}
#[cfg(all(target_arch = "aarch64", feature = "aarch64_simd"))]
unsafe impl Castable for aarch64::int8x8_t {}
#[cfg(all(target_arch = "aarch64", feature = "aarch64_simd"))]
unsafe impl Castable for aarch64::int8x8x2_t {}
#[cfg(all(target_arch = "aarch64", feature = "aarch64_simd"))]
unsafe impl Castable for aarch64::int8x8x3_t {}
#[cfg(all(target_arch = "aarch64", feature = "aarch64_simd"))]
unsafe impl Castable for aarch64::int8x8x4_t {}
#[cfg(all(target_arch = "aarch64", feature = "aarch64_simd"))]
unsafe impl Castable for aarch64::poly16x4_t {}
#[cfg(all(target_arch = "aarch64", feature = "aarch64_simd"))]
unsafe impl Castable for aarch64::poly16x4x2_t {}
#[cfg(all(target_arch = "aarch64", feature = "aarch64_simd"))]
unsafe impl Castable for aarch64::poly16x4x3_t {}
#[cfg(all(target_arch = "aarch64", feature = "aarch64_simd"))]
unsafe impl Castable for aarch64::poly16x4x4_t {}
#[cfg(all(target_arch = "aarch64", feature = "aarch64_simd"))]
unsafe impl Castable for aarch64::poly16x8_t {}
#[cfg(all(target_arch = "aarch64", feature = "aarch64_simd"))]
unsafe impl Castable for aarch64::poly16x8x2_t {}
#[cfg(all(target_arch = "aarch64", feature = "aarch64_simd"))]
unsafe impl Castable for aarch64::poly16x8x3_t {}
#[cfg(all(target_arch = "aarch64", feature = "aarch64_simd"))]
unsafe impl Castable for aarch64::poly16x8x4_t {}
#[cfg(all(target_arch = "aarch64", feature = "aarch64_simd"))]
unsafe impl Castable for aarch64::poly64x1_t {}
#[cfg(all(target_arch = "aarch64", feature = "aarch64_simd"))]
unsafe impl Castable for aarch64::poly64x1x2_t {}
#[cfg(all(target_arch = "aarch64", feature = "aarch64_simd"))]
unsafe impl Castable for aarch64::poly64x1x3_t {}
#[cfg(all(target_arch = "aarch64", feature = "aarch64_simd"))]
unsafe impl Castable for aarch64::poly64x1x4_t {}
#[cfg(all(target_arch = "aarch64", feature = "aarch64_simd"))]
unsafe impl Castable for aarch64::poly64x2_t {}
#[cfg(all(target_arch = "aarch64", feature = "aarch64_simd"))]
unsafe impl Castable for aarch64::poly64x2x2_t {}
#[cfg(all(target_arch = "aarch64", feature = "aarch64_simd"))]
unsafe impl Castable for aarch64::poly64x2x3_t {}
#[cfg(all(target_arch = "aarch64", feature = "aarch64_simd"))]
unsafe impl Castable for aarch64::poly64x2x4_t {}
#[cfg(all(target_arch = "aarch64", feature = "aarch64_simd"))]
unsafe impl Castable for aarch64::poly8x16_t {}
#[cfg(all(target_arch = "aarch64", feature = "aarch64_simd"))]
unsafe impl Castable for aarch64::poly8x16x2_t {}
#[cfg(all(target_arch = "aarch64", feature = "aarch64_simd"))]
unsafe impl Castable for aarch64::poly8x16x3_t {}
#[cfg(all(target_arch = "aarch64", feature = "aarch64_simd"))]
unsafe impl Castable for aarch64::poly8x16x4_t {}
#[cfg(all(target_arch = "aarch64", feature = "aarch64_simd"))]
unsafe impl Castable for aarch64::poly8x8_t {}
#[cfg(all(target_arch = "aarch64", feature = "aarch64_simd"))]
unsafe impl Castable for aarch64::poly8x8x2_t {}
#[cfg(all(target_arch = "aarch64", feature = "aarch64_simd"))]
unsafe impl Castable for aarch64::poly8x8x3_t {}
#[cfg(all(target_arch = "aarch64", feature = "aarch64_simd"))]
unsafe impl Castable for aarch64::poly8x8x4_t {}
#[cfg(all(target_arch = "aarch64", feature = "aarch64_simd"))]
unsafe impl Castable for aarch64::uint16x4_t {}
#[cfg(all(target_arch = "aarch64", feature = "aarch64_simd"))]
unsafe impl Castable for aarch64::uint16x4x2_t {}
#[cfg(all(target_arch = "aarch64", feature = "aarch64_simd"))]
unsafe impl Castable for aarch64::uint16x4x3_t {}
#[cfg(all(target_arch = "aarch64", feature = "aarch64_simd"))]
unsafe impl Castable for aarch64::uint16x4x4_t {}
#[cfg(all(target_arch = "aarch64", feature = "aarch64_simd"))]
unsafe impl Castable for aarch64::uint16x8_t {}
#[cfg(all(target_arch = "aarch64", feature = "aarch64_simd"))]
unsafe impl Castable for aarch64::uint16x8x2_t {}
#[cfg(all(target_arch = "aarch64", feature = "aarch64_simd"))]
unsafe impl Castable for aarch64::uint16x8x3_t {}
#[cfg(all(target_arch = "aarch64", feature = "aarch64_simd"))]
unsafe impl Castable for aarch64::uint16x8x4_t {}
#[cfg(all(target_arch = "aarch64", feature = "aarch64_simd"))]
unsafe impl Castable for aarch64::uint32x2_t {}
#[cfg(all(target_arch = "aarch64", feature = "aarch64_simd"))]
unsafe impl Castable for aarch64::uint32x2x2_t {}
#[cfg(all(target_arch = "aarch64", feature = "aarch64_simd"))]
unsafe impl Castable for aarch64::uint32x2x3_t {}
#[cfg(all(target_arch = "aarch64", feature = "aarch64_simd"))]
unsafe impl Castable for aarch64::uint32x2x4_t {}
#[cfg(all(target_arch = "aarch64", feature = "aarch64_simd"))]
unsafe impl Castable for aarch64::uint32x4_t {}
#[cfg(all(target_arch = "aarch64", feature = "aarch64_simd"))]
unsafe impl Castable for aarch64::uint32x4x2_t {}
#[cfg(all(target_arch = "aarch64", feature = "aarch64_simd"))]
unsafe impl Castable for aarch64::uint32x4x3_t {}
#[cfg(all(target_arch = "aarch64", feature = "aarch64_simd"))]
unsafe impl Castable for aarch64::uint32x4x4_t {}
#[cfg(all(target_arch = "aarch64", feature = "aarch64_simd"))]
unsafe impl Castable for aarch64::uint64x1_t {}
#[cfg(all(target_arch = "aarch64", feature = "aarch64_simd"))]
unsafe impl Castable for aarch64::uint64x1x2_t {}
#[cfg(all(target_arch = "aarch64", feature = "aarch64_simd"))]
unsafe impl Castable for aarch64::uint64x1x3_t {}
#[cfg(all(target_arch = "aarch64", feature = "aarch64_simd"))]
unsafe impl Castable for aarch64::uint64x1x4_t {}
#[cfg(all(target_arch = "aarch64", feature = "aarch64_simd"))]
unsafe impl Castable for aarch64::uint64x2_t {}
#[cfg(all(target_arch = "aarch64", feature = "aarch64_simd"))]
unsafe impl Castable for aarch64::uint64x2x2_t {}
#[cfg(all(target_arch = "aarch64", feature = "aarch64_simd"))]
unsafe impl Castable for aarch64::uint64x2x3_t {}
#[cfg(all(target_arch = "aarch64", feature = "aarch64_simd"))]
unsafe impl Castable for aarch64::uint64x2x4_t {}
#[cfg(all(target_arch = "aarch64", feature = "aarch64_simd"))]
unsafe impl Castable for aarch64::uint8x16_t {}
#[cfg(all(target_arch = "aarch64", feature = "aarch64_simd"))]
unsafe impl Castable for aarch64::uint8x16x2_t {}
#[cfg(all(target_arch = "aarch64", feature = "aarch64_simd"))]
unsafe impl Castable for aarch64::uint8x16x3_t {}
#[cfg(all(target_arch = "aarch64", feature = "aarch64_simd"))]
unsafe impl Castable for aarch64::uint8x16x4_t {}
#[cfg(all(target_arch = "aarch64", feature = "aarch64_simd"))]
unsafe impl Castable for aarch64::uint8x8_t {}
#[cfg(all(target_arch = "aarch64", feature = "aarch64_simd"))]
unsafe impl Castable for aarch64::uint8x8x2_t {}
#[cfg(all(target_arch = "aarch64", feature = "aarch64_simd"))]
unsafe impl Castable for aarch64::uint8x8x3_t {}
#[cfg(all(target_arch = "aarch64", feature = "aarch64_simd"))]
unsafe impl Castable for aarch64::uint8x8x4_t {}

#[cfg(target_arch = "x86")]
unsafe impl Castable for x86::__m128i {}
#[cfg(target_arch = "x86")]
unsafe impl Castable for x86::__m128 {}
#[cfg(target_arch = "x86")]
unsafe impl Castable for x86::__m128d {}
#[cfg(target_arch = "x86")]
unsafe impl Castable for x86::__m256i {}
#[cfg(target_arch = "x86")]
unsafe impl Castable for x86::__m256 {}
#[cfg(target_arch = "x86")]
unsafe impl Castable for x86::__m256d {}

#[cfg(target_arch = "x86_64")]
unsafe impl Castable for x86_64::__m128i {}
#[cfg(target_arch = "x86_64")]
unsafe impl Castable for x86_64::__m128 {}
#[cfg(target_arch = "x86_64")]
unsafe impl Castable for x86_64::__m128d {}
#[cfg(target_arch = "x86_64")]
unsafe impl Castable for x86_64::__m256i {}
#[cfg(target_arch = "x86_64")]
unsafe impl Castable for x86_64::__m256 {}
#[cfg(target_arch = "x86_64")]
unsafe impl Castable for x86_64::__m256d {}
