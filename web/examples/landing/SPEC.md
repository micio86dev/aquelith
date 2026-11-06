# Landing site specification (shared by every arm of the speed protocol)

Build a static site with three pages. Serve it so that each route opens its own page.

## Pages

1. Route `/`
   - Document title: `AQELITH: one language for agents`
   - Heading level 1: `Build software faster with one language`
   - Paragraph: `AQELITH is a language made for coding agents. Every program is checked before it runs.`
   - Link to `/how` with the text `How it works`
   - Link to `/start` with the text `Get started`

2. Route `/how`
   - Document title: `How AQELITH works`
   - Heading level 1: `How it works`
   - Heading level 2: `Checked by the compiler`
   - Paragraph: `Each change is checked with stable diagnostic codes an agent can read.`
   - Link to `/` with the text `Back home`

3. Route `/start`
   - Document title: `Get started with AQELITH`
   - Heading level 1: `Get started`
   - Paragraph: `Connect your coding agent with the guide in the docs folder.`
   - Link to `/` with the text `Back home`

## Acceptance

- Every route answers with its page, and its heading is the one above.
- Every link answers with status 200.
- The page loads no script.
- Text is shown exactly as written; a `<` or `&` in a text must display as itself.
- Every page links one stylesheet, `/style.css`, in its head; it answers with status 200 and type `text/css`.
- Styles change how the page looks, never its text, headings or links.

Styles are one stylesheet written by the author. Images, lists and any other content are out of scope for this spec.
