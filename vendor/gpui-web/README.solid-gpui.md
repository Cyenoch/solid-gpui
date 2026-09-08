# Solid GPUI Web platform patch

Source: gpui-pre-web 0.3.3 from crates.io. The upstream Apache-2.0 license is
retained in LICENSE-APACHE. This copy adjusts window background handling:
`set_background_appearance` records the requested appearance and configures the
existing WGPU renderer's transparency support. `background_appearance` returns
that state so GPUI clears transparent windows correctly.

This enables GPUI-rendered foreground UI over a separate browser GPU background.
The default window remains opaque. No browser UI renderer is substituted.

ResizeObserver uses contentRect for logical CSS bounds and the observed physical
width to compute render scale. Device emulation can report physical pixels at the
real screen ratio while devicePixelRatio is overridden; deriving logical size from
that overridden ratio incorrectly renders the interface at half size.
