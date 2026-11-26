# Third-Party Notices for PULSE

This file documents the licenses of all direct dependencies of PULSE (`pulse_manager` v0.2.11).
All dependencies are available under permissive open-source licenses (MIT or Apache-2.0).

## Direct Dependencies (6 crates)

| Crate | Version | License | Repository |
|-------|---------|---------|------------|
| `eframe` | 0.27.2 | MIT OR Apache-2.0 | https://github.com/emilk/egui |
| `egui` | 0.27.2 | MIT OR Apache-2.0 | https://github.com/emilk/egui |
| `egui_plot` | 0.27.2 | MIT OR Apache-2.0 | https://github.com/emilk/egui |
| `egui_extras` | 0.27.2 | MIT OR Apache-2.0 | https://github.com/emilk/egui |
| `sysinfo` | 0.30.12 | MIT | https://github.com/GuillaumeGomez/sysinfo |
| `windows-sys` | 0.52.0 | MIT OR Apache-2.0 | https://github.com/microsoft/windows-rs |

## License Texts

### MIT License (sysinfo, and applicable to MIT options above)
```
Copyright (c) <year> <copyright holders>

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

### Apache License 2.0 (eframe, egui, egui_extras, egui_plot, windows-sys)
```
                                 Apache License
                           Version 2.0, January 2004
                        http://www.apache.org/licenses/

TERMS AND CONDITIONS FOR USE, REPRODUCTION, AND DISTRIBUTION

1. Definitions.
   "License" shall mean the terms and conditions for use, reproduction,
   and distribution as defined by Sections 1 through 9 of this document.

2. Grant of Copyright License. Subject to the terms and conditions of
   this License, each Contributor hereby grants to You a perpetual,
   worldwide, non-exclusive, no-charge, royalty-free, irrevocable
   copyright license to reproduce, prepare Derivative Works of,
   publicly display, publicly perform, sublicense, and distribute the
   Work and such Derivative Works in Source or Object form.

3. Grant of Patent License. Subject to the terms and conditions of
   this License, each Contributor hereby grants to You a perpetual,
   worldwide, non-exclusive, no-charge, royalty-free, irrevocable
   (except as stated in this section) patent license to make, have made,
   use, offer to sell, sell, import, and otherwise transfer the Work,
   where such license applies only to those patent claims licensable
   by such Contributor that are necessarily infringed by their
   Contribution(s) alone.

4. Redistribution. You may reproduce and distribute copies of the
   Work or Derivative Works thereof in any medium, with or without
   modifications, and in Source or Object form, provided that You
   meet the following conditions:
   (a) You must give any other recipients of the Work or Derivative
       Works a copy of this License; and
   (b) You must cause any modified files to carry prominent notices
       stating that You changed the files; and
   (c) You must retain, in the Source form of any Derivative Works
       that You distribute, all copyright, patent, trademark, and
       attribution notices from the Source form of the Work; and
   (d) If the Work includes a "NOTICE" text file as part of its
       distribution, then any Derivative Works that You distribute must
       include a readable copy of the NOTICE file.

5. Submission of Contributions. Unless You explicitly state otherwise,
   any Contribution intentionally submitted for inclusion in the Work
   by You to the Licensor shall be under the terms and conditions of
   this License, without any additional terms or conditions.

6. Trademarks. This License does not grant permission to use the trade
   names, trademarks, service marks, or product names of the Licensor,
   except as required for reasonable and customary use in describing the
   origin of the Work and reproducing the content of the Work.

7. Disclaimer of Warranty. Unless required by applicable law or agreed
   to in writing, the Licensor provides the Work (and each Contributor
   provides its Contribution) on an "AS IS" BASIS, WITHOUT WARRANTIES
   OR CONDITIONS OF ANY KIND, either express or implied, including,
   without limitation, warranties or conditions of MERCHANTABILITY,
   FITNESS FOR A PARTICULAR PURPOSE OR NONINFRINGEMENT.

8. Limitation of Liability. In no event and under no legal theory,
   whether in tort (including negligence), contract, or otherwise,
   shall any Contributor be liable to You for damages, including any
   direct, indirect, special, incidental, or consequential damages
   arising out of or related to this License.

9. Accepting Warranty or Additional Liability. While redistributing
   the Work or Derivative Works thereof, You may choose to offer,
   and charge a fee for, acceptance of support, warranty, indemnity,
   or other liability obligations in the event You act as an agent for
   another. Notwithstanding the foregoing, You are not required to
   accept such obligations.
```

## Bundled Fonts (SIL Open Font License 1.1)

PULSE embeds two open-source typefaces in the executable for a consistent UI on every
platform. Both are distributed under the SIL Open Font License, Version 1.1.

| Font | File | Reserved Name | Copyright | License |
|------|------|---------------|-----------|---------|
| Inter | `assets/fonts/Inter-Regular.ttf` | Inter | Copyright 2016-2020 The Inter Project Authors (https://github.com/rsms/inter) | SIL OFL 1.1 |
| JetBrains Mono | `assets/fonts/JetBrainsMono-Regular.ttf` | JetBrains Mono | Copyright 2020 The JetBrains Mono Project Authors (https://github.com/JetBrains/JetBrainsMono) | SIL OFL 1.1 |

The complete license text for both fonts is bundled verbatim as
`assets/fonts/Inter-JetBrainsMono-OFL.txt` and embedded into the binary at build time.

### SIL Open Font License 1.1 - summary of terms

- The fonts and derivative works may be used, studied, modified and redistributed freely,
  including commercially, subject to the conditions below.
- Modified versions must not use the Reserved Font Names (Inter, JetBrains Mono) without
  explicit written permission from the respective copyright holders.
- The fonts may be bundled, embedded and/or redistributed with software, provided that
  this license and the copyright notices accompany every copy.
- The fonts are provided "AS IS", without warranty of any kind.
- The full legal text is authoritative and is included in the OFL file named above.

These fonts are licensed, not sold. No proprietary Microsoft font files are bundled.

## System Font Fallback

If an embedded font payload ever fails validation, PULSE falls back to Windows system
fonts (`segoeui.ttf` and `consola.ttf`) referenced by filename at runtime. Those are
proprietary Microsoft fonts and are never bundled or redistributed.