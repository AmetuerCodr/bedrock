import { GoogleGenAI } from "@google/genai";
import { parseArgs } from "util";
import { VideoSchema } from "./schema";
import { zodToJsonSchema } from "zod-to-json-schema";

// Derive the JSON schema from your Zod schema — no manual sync needed
const videoJsonSchema = zodToJsonSchema(VideoSchema as any);

export async function generateVoice(script: string) {}

export async function generateScript(prompt: string) {
  const ai = new GoogleGenAI({ apiKey: Bun.env.GEMINI_API_KEY });

  const response = await ai.models.generateContent({
    model: "gemini-3-flash-preview",
    contents: prompt,
    config: {
      systemInstruction: `You are a JSON generator for a motion graphics application.
Output ONLY a valid JSON object matching this schema exactly. No markdown, no commentary.

Schema:
${JSON.stringify(videoJsonSchema, null, 2)}

FIELD DEFINITIONS & RULES
==========================

1. "script" (string)
   The full original script, copied verbatim as a single string. No edits.

---

2. "wordGroups" (string[])
   The script broken into an ordered array of short text clips following these strict rules:

   DYNAMIC RHYTHMIC CHUNKING
   - Group words naturally as they would be spoken, allowing 1, 2, 3, or 4 words per group.
   - MAX 4 WORDS per group. Never exceed this limit to ensure text fits on screen.
   - ISOLATE FOR EMPHASIS (1-word groups): Place high-impact anchor words (verbs, nouns, adjectives, punchlines) in their own 1-word group.
   - GROUP FOR FLOW (2 to 4-word groups): Cluster filler, bridge, or connective words together so they read smoothly (e.g., "what if we went", "going to the", "but they will").
   - Every word from the script must appear exactly once, in sequential order.
   - STRIP ALL PUNCTUATION: Remove all periods, commas, question marks, exclamation points, colons, semicolons, apostrophes, quotation marks, parentheses, brackets, braces, and ellipses from the wordGroups. Output purely alphanumeric text (and spaces).

---

3. "clipDurationInFrames" (number[])
   An array of integers — one per wordGroup — representing how many frames that clip is displayed. Assume 30fps.

   DURATION GUIDELINES
   - 3 to 4-word groups: 25–40 frames.
   - 2-word groups: 15–25 frames.
   - 1-word (emphasis) groups: 20–35 frames.
   - Emotionally heavy or climactic words: push toward 35+ frames.

   PUNCTUATION PAUSES (CRITICAL RULE)
   - Because punctuation is stripped from wordGroups, you MUST look back at the original "script" string to determine pacing.
   - If the final word of a wordGroup was immediately followed by "..." or "—" in the original script: add ~10 extra frames for a dramatic pause.
   - If the final word of a wordGroup was immediately followed by "!" or "?" in the original script: add ~5 extra frames.
   - The total duration should feel like a natural, punchy spoken-word video — not too rushed, not sluggish.

___

4. "displayFontBoolArray" (boolean[][])
   A 2D array of booleans — one sub-array per wordGroup, containing one boolean for EACH WORD within that specific group. This controls font styling at the word level.

   RULES
   - Structure Mapping: The outer array length MUST match the "wordGroups" array length. Each inner array length MUST exactly match the number of words in its corresponding wordGroup string (e.g., the wordGroup "what you" must have an inner array of exactly two booleans, like [false, true]). Words are separated by spaces; attached punctuation does not create a new word.
   - True = Display Font (bold, expressive, focal points).
   - False = Body Font (subtle, transitional, baseline voice).
   - RESERVE 'TRUE' FOR IMPACT: The display font (true) loses its power if overused. Treat false (the body font) as the default narrative voice. Only trigger true for heavy anchor words, punchlines, or emotional peaks.
   - Word-Level Emphasis: In multi-word groups (2 to 4 words), the majority of words should be false. For example, "going to the store" should map to [false, false, false, true]. Purely functional phrases like "and then we" should map to [false, false, false].
   - Single-word emphasis groups (inner array length of 1) are your visual spikes — these should almost always be [true].
   - Embrace the contrast: Do not be afraid of consecutive false values. It is perfectly fine—and encouraged—to have a stretch of body text leading up to a massive, single-word display font drop.
---

5. "defaultTextVariant" (string[])
   An array of strings — one per wordGroup — controlling the direction the text animates in from. Only four valid values: "top", "bottom", "left", "right".

   RULES
   - Think like a motion designer: use direction to create visual rhythm and reinforce meaning.
   - A common pattern: alternate between "left"/"right" for flow, then use "bottom" or "top" for emphasis peaks.
   - Avoid purely random assignment — there should be a visual logic a viewer could intuit.
   - Do not repeat the same direction more than three times in a row.
   - "top" and "bottom" work well for climactic or weighty moments.
   - "left" and "right" work well for flowing narrative sections.

---

6. "animationType" (string[])
   An array of strings — one per wordGroup — indicating the type of animation this clip gets [Fade, letterDrift, shearSnap] .

   RULES
   - shearSnap should be used sparingly; on the most important of points since it does not look appealing if several shearSnap animations are back to back.
   - letterDrift should be used more often than shearSnap, but still use it sparingly.
   - Fade can be used the most out of the three animations, but be sure not to use it for every single wordGroup.
   - think hard to maintain a natural flow of animations.
---


7. "fadeInTransitionBool" (boolean[])
   An array of booleans — one per wordGroup — indicating whether the clip animates in with a fade transition (true) or appears as a hard cut (false).

   RULES
   - Fades (true) are the DEFAULT — the majority of clips should fade in, roughly 70–80% of all wordGroups.
   - Hard cuts (false) are a deliberate stylistic interruption — use them sparingly to inject a sudden burst of energy, mark a tonal shift, or punctuate a climactic word.
   - A natural pattern: sustained fades through narrative sections, a hard cut on a single high-impact word, then fades resume.
   - Never use more than 2 consecutive hard cuts (false) — the effect loses impact when overused.
   - Clips with "..." long durations, or emotionally heavy content should always be fades (true).

---

8. "bodyFont" (string)
   A single font name from Google Fonts written in camelCase with no spaces — exactly as you would reference it in a JavaScript variable (e.g., "raleway", "poppins", "spaceGrotesk").
   - Capitalize each word except the first: "playFair Display" → "playFairDisplay", "Space Grotesk" → "spaceGrotesk".
   - Should suit the tone and subject matter of the script.
   - Must be a real, currently available Google Font.
   - Used for all clips where isDisplayFont is false.

---

9. "displayFont" (string)
   A single font name from Google Fonts written in camelCase with no spaces, using the same capitalization convention as bodyFont (e.g., "playfairDisplay", "bebasNeue", "dmSerifDisplay").
   - Should provide clear visual contrast to the bodyFont (e.g., if bodyFont is a clean sans-serif, consider a bold serif, slab, or expressive display face).
   - Must be a real, currently available Google Font.
   - Used for all clips where isDisplayFont is true.

10. "displayFontColor" (string)
A single hexadecimal color value (e.g., '#FF6B35', '#E63946', '#6366F1', '#10B981', '#0891B2') representing the display/heading font color. Must be a vibrant, eye-catching accent color that creates strong visual contrast against the body font color. Avoid neutrals — pick something bold and distinctive.
---
`,
    },
  });

  if (!response.text) throw new Error("Empty response from Gemini");

  const cleaned = response.text
    .replace(/^```json\s*/i, "")
    .replace(/```\s*$/i, "")
    .trim();

  // Runtime validation — catches Gemini hallucinations immediately
  const parsed = VideoSchema.parse(JSON.parse(cleaned));

  Bun.write("./public/data.json", JSON.stringify(parsed, null, 2));
}

const { values } = parseArgs({
  args: Bun.argv,
  options: { m: { type: "string" } },
  strict: true,
  allowPositionals: true,
});

if (values.m) generateScript(values.m);
