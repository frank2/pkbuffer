# Changelog

## 0.4.1
### Features
* added a dynamic search feature to buffers. Now you can search for a dynamic pattern of bytes, see the docs for more details!

## 0.4.0
### Bugfixes
* fixed the undefined behavior of `ref_to_bytes` and similar functions by taking inspiration from [bytemuck](https://crates.io/crate/bytemuck) and actively preventing undefined behavior, thanks to @[repnop](https://github.com/repnop) for reporting this! this changes the generic signatures to have the trait `Castable` (e.g., `T: Castable`) in multiple functions. see the docs for more details!
### Features
* objects acquired by `get_ref` and similar functions now require the `Castable` trait. this guarantees the safety of types acquired from byte buffers, see the docs for more details.
* with the addition of `Castable` comes alignment awareness. however, there are some cases (such as with PE files) where reading unaligned structures is necessary. as a result, `Buffer::get_ref_unaligned`, `Buffer::force_get_ref` and similar functions have been added to deal with unaligned objects.
* the `Castable` trait can be derived for any struct given it meets the requirements, see the docs for more details.

## 0.3.0
**This version makes major changes!** The `Buffer` struct is now a trait, and the struct has been converted to `PtrBuffer`. This simplifies the code and unifies the featureset without having to repeat code all over the place. Additionally, it gives the user the ability to define their own `Buffer` object.

### Bugfixes
* `IntoIter` was returning references when it should be returning values, this is fixed.
* `Error` now implements `Send` and `Sync`.
### Features
* `Buffer` is now a trait, which allows the featureset of both `VecBuffer` and the new `PtrBuffer` to be unified.
* `Error` now makes use of the `into()` function with regards to `std::io::Error`.

## 0.2.0
### Bugfixes
* ```Buffer``` object was not getting updated on clone in ```VecBuffer```. That's now fixed.
### Features
* ```Buffer::search``` and similar functions now return an iterator to all search results, including no search results.

## 0.1.0
### Features
* package released!
* created a buffer object that points to arbitrary memory locations via ```u8``` pointer
* created a vector-backed buffer object that points at the vector with extra vector features
