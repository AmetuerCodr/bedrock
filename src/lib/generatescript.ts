import { GoogleGenAI } from "@google/genai";
import { ElevenLabsClient } from "@elevenlabs/elevenlabs-js";
import { parseArgs } from "util";

async function scriptToVoiceOver(text: string) {
  console.log("program running");
  const elevenlabsclient = new ElevenLabsClient({
    apiKey: Bun.env.ELEVENLABS_KEY,
  });
  const id = Bun.randomUUIDv7();
  const file = Bun.file(`${id}.mp3`);
  const writer = file.writer();
  const audio: ReadableStream = await elevenlabsclient.textToSpeech.convert(
    "pqHfZKP75CvOlQylNhV4",
    {
      outputFormat: "mp3_44100_128",
      text: text,
      modelId: "eleven_multilingual_v2",
    },
  );

  if (audio) {
    const buf = await new Response(audio as ReadableStream).arrayBuffer();
    writer.write(buf);
    console.log("voiceover written to disk!");
  }
}

export async function generateScript(prompt: string) {
  //todo make the api swap models when in high demand

  // 13061 |         const errorMessage = JSON.stringify(errorBody);
  // 13062 |         if (status >= 400 && status < 600) {
  // 13063 |             const apiError = new ApiError({
  //                                      ^
  // ApiError: {"error":{"code":503,"message":"This model is currently experiencing high demand. Spikes in demand are usually temporary. Please try again later.","status":"UNAVAILABLE"}}
  //  status: 503,
  const ai = new GoogleGenAI({
    apiKey: Bun.env.GEMINI_API_KEY,
  });
  const response = await ai.models.generateContent({
    model: "gemini-2.5-flash",
    contents: prompt,
    config: {
      systemInstruction: `You are a JSON generator. Produce NO commentary, explanation, or markdown — output ONLY a single valid JSON object.

      Given a script, return an object with exactly three fields:

      {
        "script": <the full original script as a single string>,
        "wordGroups": <the script broken into a JSON array of strings, following the rules below>,
        "clipDurationInFrames": <a JSON array of integers, one per element in wordGroups, representing how long each clip should last in frames>
        "isDisplayFont": <a JSON array of boolean values, one value per element in wordGroups, reperesenting whetehr the words in that group should be a special display font (true) or whether it should be the regular font (false)>
      }

      ---

      WORD GROUP RULES (for the "wordGroups" field):
      Break the script into an array of strings using these strict rules:

      1. ALTERNATING STRUCTURE: Elements must strictly alternate between TWO-word groups and ONE-word groups. Start with a two-word group.
      2. EMPHASIS: Single-word elements must be the "anchor" word — the verb, noun, or adjective carrying the most meaning in that phrase.
      3. FLOW: Two-word elements should contain functional/supporting words (e.g. the, is, with, they, what, how).
      4. Do not skip or combine steps — every word in the script must appear in the array in order.

      ---

      CLIP DURATION RULES (for the "clipDurationInFrames" field):
      - The array must be the same length as "wordGroups" — one integer per group.
      - Single-word (emphasis) clips: typically 20–35 frames. Use more frames for emotionally heavy words.
      - Two-word clips: typically 15–25 frames.
      - Punctuation like "..." or "!" signals a pause — increase that clip's duration by ~10 frames.
      - Use your judgment to make the pacing feel natural and dramatic.

      ---

      EXAMPLE:

      Input script: "People will forget what you said... but people will never forget how you made them feel."

      Expected output:
      {
        "script": "People will forget what you said... but people will never forget how you made them feel.",
        "wordGroups": [
          "People will",
          "forget",
          "what you",
          "said...",
          "but people",
          "will never",
          "forget",
          "how you",
          "made them",
          "feel"
        ],
        "clipDurationInFrames": [18, 25, 18, 35, 18, 18, 30, 18, 18, 35]
      }`,
    },
  });
  if (response.text) {
    const cleanedContents = response.text
      .replace(/^```json\s*/i, "")
      .replace(/```\s*$/i, "")
      .trim();
    Bun.write("./public/data.json", cleanedContents);
    // scriptToVoiceOver(response.text);
  }
}
const { values } = parseArgs({
  args: Bun.argv,
  options: {
    m: {
      type: "string",
    },
  },
  strict: true,
  allowPositionals: true,
});
const message = values.m?.toString();
if (message) generateScript(message);
