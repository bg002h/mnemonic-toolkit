# Filter smoke test

This fixture exists so Phase 0 can verify the pandoc filter pipeline
behaves correctly in **both** render paths:

- `make md` must produce a styled blockquote and strip every `\index{}`
  / raw-LaTeX construct.
- `make pdf-docker` must emit a real `\begin{primerbox}` and register
  the index entry on the page-numbered index page.

The m-format star\index{m-format star} is the unit covered.

:::primer
This fenced div should render as a boxed sidebar in the PDF and as a
`> **Background.** ...` blockquote in the markdown.
:::

End of smoke fixture.
