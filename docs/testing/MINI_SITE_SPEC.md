# Spec: a four-page product site for a small note-taking tool

Status: test input for the agent trials of 2026-11-06. The site it describes is not part of this repository and is not implemented here.

Build a static site with four pages. Each route opens its own page.

1. Route `/`
   - Document title: `Notely: notes for developers`
   - Heading level 1: `Notes that stay in your repo`
   - Paragraph: `Notely keeps plain-text notes next to your code and checks them in review.`
   - Link to `/features` with the text `See the features`
   - Link to `/pricing` with the text `Pricing`

2. Route `/features`
   - Document title: `Features of Notely`
   - Heading level 1: `Features`
   - Heading level 2: `Plain text`
   - Paragraph: `Every note is a text file you can read and diff.`
   - Heading level 2: `Review`
   - Paragraph: `Notes are reviewed like code, in the same pull request.`
   - Link to `/` with the text `Back home`
   - Link to `/contact` with the text `Contact us`

3. Route `/pricing`
   - Document title: `Pricing for Notely`
   - Heading level 1: `Pricing`
   - Paragraph: `Notely is free for open source projects.`
   - Link to `/contact` with the text `Ask about a team plan`
   - Link to `/` with the text `Back home`

4. Route `/contact`
   - Document title: `Contact Notely`
   - Heading level 1: `Contact`
   - Paragraph: `Write to the maintainers through the project issue tracker.`
   - Link to `/` with the text `Back home`

## Acceptance

- Link one stylesheet, `/style.css`, from every page, and provide the stylesheet as `style.css`.
- Every route shows its page with the heading above.
- Every link leads to the route named in it.
- The site builds and passes its checks.
- Deliverables: the source file(s) in the run's working folder, and the built site in `dist/` (including `style.css`).
