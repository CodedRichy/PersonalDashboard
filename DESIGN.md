# Dashboard design

Reference: Dribbble shot 3 ("My Design Tasks"): light, airy, rounded cards on a soft pastel wash, a ranked task list beside a schedule.

## Colour (OKLCH tokens in `src/tokens.css`)
- Peach `--due`: due soon or overdue.
- Orchid `--work` (hue 320): unpushed work. The vault's colour notes call hue 270-300 the AI "slop zone", so the lavender from the reference was moved to 320.
- Slate `--stale`: stale projects. Neutral on purpose, it is not urgent.
- Mint `--clear`: all clear.
- Colour carries urgency before the text is read. This is the one deliberate risk: tinted cards instead of priority badges.

## Type
Plus Jakarta Sans Variable, bundled with @fontsource (works offline under the strict CSP). Not Inter. Stat numerals are large and tabular. Sizes use `clamp()`.

## Shape
22px radius cards, hairline border, soft two-layer shadow. No backdrop blur (the vault lists blur-on-everything as a tell); the glass feel comes from a translucent white card over the gradient wash.

## Layout
- Header "My Day" with the date.
- Stat trio, asymmetric (the first stat is wider) so it does not read as three equal cards.
- Centre column: ranked task cards, each with a one-line reason and a pill chip.
- Right column: Schedule, all deadlines in date order.
- Bottom-right slot reserved for the pet. Empty in v0.

## Motion
Cards rise in over 0.35s. Nothing bounces. Disabled under reduced motion.

## Out of scope
Dark mode, the pet, charts.
