# Third-party licenses

The `vyges-openroad` binary statically links the following components. Their versions track
the pins in [`vyges-tools/odb-sys`](https://github.com/vyges-tools/odb-sys)
(`openroad-pin.yaml`); this file is included in every release archive for binary-distribution
compliance. Full upstream license texts are at the linked repositories.

| Component | Version (pinned) | License | Upstream |
|-----------|------------------|---------|----------|
| OpenROAD OpenDB (`libodb`) | pinned SHA (26Q3) | BSD-3-Clause | https://github.com/The-OpenROAD-Project/OpenROAD |
| fmt        | 11.0.2      | MIT        | https://github.com/fmtlib/fmt |
| spdlog     | 1.15.3      | MIT        | https://github.com/gabime/spdlog |
| Abseil     | 20250127.0  | Apache-2.0 | https://github.com/abseil/abseil-cpp |
| zlib (dynamic) | system  | zlib       | https://zlib.net |

`vyges-openroad` itself is Apache-2.0 (see `LICENSE`). OpenROAD-derived code (`libodb`) is
BSD-3-Clause; its copyright notice and license text are reproduced per that license — see
`NOTICE` and the `OPENROAD-LICENSE-BSD3.txt` shipped in the `vyges-tools/odb-sys` libodb bundle.
