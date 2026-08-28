# Runtime font command

Type: task
Status: claimed

Add root-only `COMMAND_LOAD_FONT = 29` with bounded absolute-path validation,
background file loading, GPUI `TextSystem::add_fonts` registration, family-name
return, and bilateral protocol coverage. The command must preserve the
registration-before-first-layout invariant documented in the feature spec.
