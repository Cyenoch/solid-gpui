# Solid GPUI browser transparency patch

Source: gpui-pre-wgpu 0.3.3 from crates.io, Apache-2.0 (LICENSE-APACHE).

The only local change selects PreMultiplied for transparent BrowserWebGpu
surfaces. wgpu 29.0.4's browser `Surface::get_capabilities` advertises only Opaque,
but its `Surface::configure` explicitly maps PreMultiplied to WebGPU's supported
`premultiplied` canvas alpha mode. Relying on the incomplete capability list makes
GPUI silently create an opaque canvas. Other backends retain capability selection.
