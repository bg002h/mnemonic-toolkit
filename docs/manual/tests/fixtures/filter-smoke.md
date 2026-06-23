# Filter smoke test

This fixture exists so Phase 0 can verify the pandoc filter pipeline
behaves correctly in both render paths. The markdown render must
strip raw-LaTeX inlines and produce a styled blockquote. The PDF
render must emit boxed sidebars and register page-numbered index
entries.

The m-format constellation\index{m-format constellation} is the unit covered.

:::primer
This fenced div must render as a boxed sidebar in the PDF and as a
"Background." blockquote in the markdown.
:::

The whole-include fence below exercises include-transcript.lua's no-`lines=`
(whole-file) path and carries a >64-char `xpub6…` run so the wrap-long-code
filter has a non-vacuous chunk to split in the LaTeX/PDF render.

```{.text include="include-whole-sample.out"}
PLACEHOLDER — generated from tests/fixtures/include-whole-sample.out at build
```

End of smoke fixture.
