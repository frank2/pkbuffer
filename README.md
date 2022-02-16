# PKBuffer
```PKBuffer``` is a simple but flexible buffer library that allows you to cast Rust objects onto a byte buffer. It was born of multiple memory parsing scenarios such as executable header parsing and game ROM memory mapping. Its name comes from the magic that is casting arbitrary structure definitions at byte buffers and the fact that I was reverse engineering the game [EarthBound](https://en.wikipedia.org/wiki/EarthBound) at the time. 

You can read the documentation [here](https://docs.rs/pkbuffer/), and see various use examples in [the test file](https://github.com/frank2/pkbuffer/blob/main/src/tests.rs).

# Changelog

## 0.1.0
### features
* package released!
* created a buffer object that points to arbitrary memory locations via ```u8``` pointer
* created a vector-backed buffer object that points at the vector with extra vector features
