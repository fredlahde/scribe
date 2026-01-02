# Third Party Licenses

Scribe includes code from third-party projects. This document provides attribution
and license information for these dependencies.

## Overview

Scribe is built with Rust and TypeScript/Vue. The complete dependency tree includes
over 460 crates and numerous npm packages. Below are the notable licenses and
attributions required by those licenses.

---

## License Summary

| License | Count | Type |
|---------|-------|------|
| Apache-2.0 OR MIT | 351 | Permissive |
| MIT | 146 | Permissive |
| Apache-2.0 OR MIT OR Zlib | 26 | Permissive |
| Unicode-3.0 | 18 | Permissive (attribution required) |
| MIT OR Unlicense | 7 | Permissive |
| Apache-2.0 | 5 | Permissive |
| MPL-2.0 | 5 | Weak copyleft (file-level) |
| BSD-3-Clause | 3 | Permissive |
| Unlicense | 2 | Public domain |
| BSL-1.0 | 2 | Permissive |
| ISC | 2 | Permissive |
| Zlib | 1 | Permissive |

---

## MPL-2.0 Licensed Dependencies

The following dependencies are licensed under the Mozilla Public License 2.0.
MPL-2.0 is a file-level copyleft license. Modifications to these specific source
files must be released under MPL-2.0, but the license does not extend to the
rest of the project.

- **cssparser** - CSS parsing library
- **cssparser-macros** - Procedural macros for cssparser
- **dtoa-short** - Fast float-to-string conversion
- **option-ext** - Option extension traits
- **selectors** - CSS selector matching

These are transitive dependencies used by the WebView rendering engine (wry/WebKit).

Full license text: https://www.mozilla.org/en-US/MPL/2.0/

---

## Unicode-3.0 Licensed Dependencies

The following ICU-related crates are licensed under the Unicode License Agreement
(Unicode-3.0). This license requires the following notice to be included:

### Unicode License

Copyright (c) 1991-2024 Unicode, Inc. All rights reserved.

UNICODE, INC. LICENSE AGREEMENT - DATA FILES AND SOFTWARE

See Terms of Use <https://www.unicode.org/copyright.html>

Permission is hereby granted, free of charge, to any person obtaining
a copy of the Unicode data files and any associated documentation
(the "Data Files") or Unicode software and any associated documentation
(the "Software") to deal in the Data Files or Software
without restriction, including without limitation the rights to use,
copy, modify, merge, publish, distribute, and/or sell copies of
the Data Files or Software, and to permit persons to whom the Data Files
or Software are furnished to do so, provided that either
(a) this copyright and permission notice appear with all copies
of the Data Files or Software, or
(b) this copyright and permission notice appear in associated
Documentation.

THE DATA FILES AND SOFTWARE ARE PROVIDED "AS IS", WITHOUT WARRANTY OF
ANY KIND, EXPRESS OR IMPLIED, INCLUDING BUT NOT LIMITED TO THE
WARRANTIES OF MERCHANTABILITY, FITNESS FOR A PARTICULAR PURPOSE AND
NONINFRINGEMENT OF THIRD PARTY RIGHTS.

**Affected crates:**
- icu_collections
- icu_locale_core
- icu_normalizer
- icu_normalizer_data
- icu_properties
- icu_properties_data
- icu_provider
- litemap
- potential_utf
- tinystr
- writeable
- yoke
- yoke-derive
- zerofrom
- zerofrom-derive
- zerotrie
- zerovec
- zerovec-derive

---

## Unlicense / Public Domain Dependencies

The following crates are released into the public domain under the Unlicense:

- **whisper-rs** - Rust bindings for whisper.cpp
- **whisper-rs-sys** - FFI bindings for whisper.cpp

Note: whisper-rs wraps whisper.cpp, which is licensed under MIT.

---

## whisper.cpp (MIT License)

The Whisper speech recognition functionality is provided by whisper.cpp.

```
MIT License

Copyright (c) 2023-2024 The ggml authors

Permission is hereby granted, free of charge, to any person obtaining a copy
of this software and associated documentation files (the "Software"), to deal
in the Software without restriction, including without limitation the rights
to use, copy, modify, merge, publish, distribute, sublicense, and/or sell
copies of the Software, and to permit persons to whom the Software is
furnished to do so, subject to the following conditions:

The above copyright notice and this permission notice shall be included in all
copies or substantial portions of the Software.

THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR
IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY,
FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE
AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER
LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM,
OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN THE
SOFTWARE.
```

---

## Tauri Framework (MIT OR Apache-2.0)

Scribe is built on the Tauri framework.

```
MIT License

Copyright (c) 2017 - Present Tauri Programme within The Commons Conservancy

Permission is hereby granted, free of charge, to any person obtaining a copy
of this software and associated documentation files (the "Software"), to deal
in the Software without restriction, including without limitation the rights
to use, copy, modify, merge, publish, distribute, sublicense, and/or sell
copies of the Software, and to permit persons to whom the Software is
furnished to do so, subject to the following conditions:

The above copyright notice and this permission notice shall be included in all
copies or substantial portions of the Software.

THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR
IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY,
FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE
AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER
LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM,
OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN THE
SOFTWARE.
```

---

## Vue.js (MIT License)

The frontend is built with Vue.js.

```
MIT License

Copyright (c) 2013-present, Yuxi (Evan) You

Permission is hereby granted, free of charge, to any person obtaining a copy
of this software and associated documentation files (the "Software"), to deal
in the Software without restriction, including without limitation the rights
to use, copy, modify, merge, publish, distribute, sublicense, and/or sell
copies of the Software, and to permit persons to whom the Software is
furnished to do so, subject to the following conditions:

The above copyright notice and this permission notice shall be included in all
copies or substantial portions of the Software.

THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR
IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY,
FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE
AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER
LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM,
OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN THE
SOFTWARE.
```

---

## SQLite (Public Domain)

Scribe uses SQLite for local history storage via the rusqlite crate.

SQLite is in the public domain. No license is required.

https://www.sqlite.org/copyright.html

---

## Other Notable Dependencies

### Audio Processing
- **cpal** (Apache-2.0) - Cross-platform audio I/O
- **rubato** (MIT) - Audio resampling

### Input Simulation  
- **enigo** (MIT) - Cross-platform input simulation

### Image Processing
- **png** (MIT OR Apache-2.0) - PNG encoding/decoding

---

## Full Dependency List

For a complete list of all dependencies and their licenses, run:

```bash
cargo license --manifest-path src-tauri/Cargo.toml
```

Or for JSON output:

```bash
cargo license --manifest-path src-tauri/Cargo.toml --json
```

---

## Contact

If you believe there is a licensing issue or missing attribution, please open
an issue at the project repository.
