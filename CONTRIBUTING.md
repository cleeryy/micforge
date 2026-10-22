# Contributing to Yeti Voice

Thanks! A few simple rules:

1. **Small, focused PRs**: one feature / one fix per PR.
2. **Tested Core**: any logic in `BlueYeti.Core` must be covered by xunit.
   Run `dotnet test` before pushing (the WPF project only builds on Windows).
3. **No binaries/drivers** in the repo: the virtual cable stays external (see README).
4. **Presets**: any new JSON preset must follow the schema (`version`,
   `highpass`, `rnnoise`, `gate`, `comp`, `eq`, `deesser`, `limiter`, `notes`)
   and pass `python3 -m json.tool`.
5. **English** for user docs; code and comments in English.
6. License: every contribution is under AGPL-3.0-only.
