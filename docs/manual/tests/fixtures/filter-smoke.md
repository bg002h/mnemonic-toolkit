# Filter smoke test

This fixture exists so Phase 0 can verify the pandoc filter pipeline
behaves correctly in both render paths. The markdown render must
strip raw-LaTeX inlines and produce a styled blockquote. The PDF
render must emit boxed sidebars and register page-numbered index
entries.

The m-format star\index{m-format star} is the unit covered.

:::primer
This fenced div must render as a boxed sidebar in the PDF and as a
"Background." blockquote in the markdown.
:::

End of smoke fixture.
