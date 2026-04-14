import { GoogleGenAI } from "@google/genai";
import { parseArgs } from "util";
import { VideoSchema } from "./schema";
import { zodToJsonSchema } from "zod-to-json-schema";

// Derive the JSON schema from your Zod schema — no manual sync needed
const videoJsonSchema = zodToJsonSchema(VideoSchema as any);

export async function generateScript(prompt: string) {
  const ai = new GoogleGenAI({ apiKey: Bun.env.GEMINI_API_KEY });

  const response = await ai.models.generateContent({
    model: "gemini-2.5-flash",
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

   ALTERNATING STRUCTURE
   - Elements MUST strictly alternate: TWO-word group → ONE-word group → TWO-word group → ONE-word group...
   - Always begin with a TWO-word group.
   - Every word from the script must appear exactly once, in order.
   - If the script ends mid-pattern (e.g., only one word remains when a two-word group is expected), place that single word alone as the final element — the alternating rule bends only at the very end.

   EMPHASIS (one-word groups)
   - The single word must be the "anchor" — the verb, noun, or adjective carrying the most semantic weight in its surrounding phrase.
   - Avoid using filler words (a, the, is, of, in, to) as solo elements.

   FLOW (two-word groups)
   - Should contain functional or supporting words that bridge meaning (e.g., "what you", "they will", "but we").
   - Punctuation stays attached to the word it follows (e.g., "said..." or "feel.").

---

3. "clipDurationInFrames" (number[])
   An array of integers — one per wordGroup — representing how many frames that clip is displayed. Assume 30fps.

   DURATION GUIDELINES
   - Two-word groups: 15–25 frames (avg ~18).
   - One-word (emphasis) groups: 20–35 frames (avg ~27).
   - Emotionally heavy or climactic words: push toward 35+ frames.
   - Clips ending with "..." or "—": add ~10 extra frames for a dramatic pause.
   - Clips ending with "!" or "?": add ~5 extra frames.
   - The total duration should feel like a natural, punchy spoken-word video — not too rushed, not sluggish.

---

4. "isDisplayFont" (boolean[])
   An array of booleans — one per wordGroup — indicating whether that clip should render in the display font (true) or the body font (false).

   RULES
   - The display font is the DISPLAY font for this video — use it for words that should stand out in clips.
   - Use false (body font) selectively for transitional, low-energy two-word bridge groups that carry little emotional weight — think of the body font as a deliberate moment of visual rest between display font bursts.
   - Single-word emphasis groups should almost always be true.
   - Do not cluster too many consecutive false values — no more than 2 body font clips in a row before returning to the display font.
   - The overall feel should be bold and expressive, with the body font used only as a subtle contrast.

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

6. "fadeInTransitionBool" (boolean[])
   An array of booleans — one per wordGroup — indicating whether the clip animates in with a fade transition (true) or appears as a hard cut (false).

   RULES
   - Fades (true) are the DEFAULT — the majority of clips should fade in, roughly 70–80% of all wordGroups.
   - Hard cuts (false) are a deliberate stylistic interruption — use them sparingly to inject a sudden burst of energy, mark a tonal shift, or punctuate a climactic word.
   - A natural pattern: sustained fades through narrative sections, a hard cut on a single high-impact word, then fades resume.
   - Never use more than 2 consecutive hard cuts (false) — the effect loses impact when overused.
   - Clips with "..." long durations, or emotionally heavy content should always be fades (true).

---

7. "bodyFont" (string)
   A single font name from Google Fonts written in camelCase with no spaces — exactly as you would reference it in a JavaScript variable (e.g., "raleway", "poppins", "spaceGrotesk").
   - Capitalize each word except the first: "playFair Display" → "playFairDisplay", "Space Grotesk" → "spaceGrotesk".
   - Should suit the tone and subject matter of the script.
   - Must be a real, currently available Google Font.
   - Used for all clips where isDisplayFont is false.

---

8. "displayFont" (string)
   A single font name from Google Fonts written in camelCase with no spaces, using the same capitalization convention as bodyFont (e.g., "playfairDisplay", "bebasNeue", "dmSerifDisplay").
   - Should provide clear visual contrast to the bodyFont (e.g., if bodyFont is a clean sans-serif, consider a bold serif, slab, or expressive display face).
   - Must be a real, currently available Google Font.
   - Used for all clips where isDisplayFont is true.

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
