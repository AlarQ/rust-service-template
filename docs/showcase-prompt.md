# Project Showcase - Setup Prompt

Use this prompt with Claude Code (or any AI assistant) inside a project repository to generate a showcase page. Copy the prompt below, adjust the bracketed instructions if needed, and run it.

---

## Prompt

```
Create a project showcase page for this repository using the template from my docs repo.

### Template
The template file is located at `./project-showcase-template.html` in this repository.

Copy it to `showcase.html` in this repository's root and edit only the config.

### What to change
Only edit the `PROJECT_CONFIG` object at the top of the `<script>` tag. Do NOT modify the CSS or the render engine below it.

### Config fields to populate

Read through the project's README, CLAUDE.md, source code, and any documentation to fill in:

1. **title** - Project name
2. **tagline** - One-sentence summary (what it does + how)
3. **status** - Current state (e.g. "In Development", "Production", "MVP", "85% Complete")
4. **about** - Array of 2-3 paragraphs:
   - What the project does and why it exists
   - Technical approach and architecture philosophy
   - Current state and what makes it interesting
5. **architecture** - Object with:
   - `description`: 1-2 sentence summary of the architecture
   - `diagram`: Mermaid.js diagram (graph TB or graph TD) showing system components and data flow. Use Mermaid syntax with nodes, edges, and styling. Show the main layers: client, API, services, data stores. Use solid arrows (-->) for primary data flow and dashed arrows (-.->)for event/message flow.
6. **techStack** - Array of `{ category, items }` groups. Common categories: Frontend, Backend, Database, Infrastructure, Testing, Monitoring. List actual technologies used, not generic terms
7. **features** - Array of 4-6 `{ icon, title, description }` cards. Pick the most impressive/differentiating features. Use a single emoji for icon. Description should be 1-2 sentences explaining what it does and why it matters
8. **challenges** - Array of 2-4 `{ title, problem, solution }` cards. Pick real technical challenges that demonstrate engineering thinking. Problem: what made it hard. Solution: the approach and tradeoffs
9. **metrics** - Array of `{ label, value, type }` items:
   - `type: "percentage"` renders a progress bar (value 0-100)
   - `type: "count"` renders a large number
   - Good metrics: feature completion %, test coverage %, number of services/endpoints/models, lines of code, production readiness %
10. **footer** - Set to `null` (page will be iframe-embedded)

### Sections are optional
Omit any config key (or set to `null`) to hide that section entirely. At minimum include: title, tagline, about, architecture (with Mermaid diagram), techStack, and features.

Note: The links section has been removed from the template.

### GitHub Pages deployment
Ensure `showcase.html` is included in the repository's GitHub Pages deployment. Check the existing GitHub Pages configuration (GitHub Actions workflow, `docs/` folder, or `gh-pages` branch) and adjust it so that `showcase.html` is served at the root of the Pages site. The page must be accessible at `https://<user>.github.io/<repo>/showcase.html`.

### Quality checklist
- About text reads well for a non-technical recruiter AND a senior engineer
- Architecture diagram (Mermaid) is accurate to the actual system and renders correctly
- Tech stack lists real dependencies, not aspirational ones
- Challenges describe genuine problems you solved, not textbook examples
- Metrics are honest (don't inflate numbers)
```

---

## Example output reference

See the Money Planner showcase for a working example:
- Source: `money-planner-showcase.html` in this repo
- Live: https://alarq.github.io/monney-planner-docs/money-planner-showcase.html
